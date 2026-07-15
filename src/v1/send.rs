//! HTTP/JSON transport every Microsoft Graph coroutine delegates to:
//! builds the authorized request and parses the JSON response, or the
//! Graph error envelope on failure.
//!
//! Microsoft Graph reference:
//! <https://learn.microsoft.com/en-us/graph/api/overview>.

use core::marker::PhantomData;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use io_http::{
    coroutine::{HttpCoroutine, HttpCoroutineState},
    rfc6750::bearer::HttpAuthBearer,
    rfc9110::{
        request::HttpRequest,
        send::{HttpSendOutput, HttpSendYield},
    },
    rfc9112::send::{Http11Send, Http11SendError},
};
use log::{debug, trace};
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use thiserror::Error;
use url::Url;

use crate::coroutine::{MsgraphCoroutine, MsgraphCoroutineState, MsgraphYield};

/// Base URL of the Microsoft Graph API, version 1.0.
pub const MSGRAPH_API_BASE: &str = "https://graph.microsoft.com/v1.0/";

/// Base path segment addressing a mailbox owner: `me` as-is, any
/// other value as `users/{id}`.
///
/// Graph accepts an explicit user id or principal name under `users/`
/// but rejects `users/me`; only the bare `me` shortcut addresses the
/// authenticated user.
pub fn user_path(user_id: &str) -> String {
    if user_id == "me" {
        String::from("me")
    } else {
        format!("users/{user_id}")
    }
}

/// Marker for endpoints that return an empty body (DELETE, sendMail,
/// send draft); deserialises from anything, including nothing.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct MsgraphNoResponse;

impl<'de> Deserialize<'de> for MsgraphNoResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _ = serde::de::IgnoredAny::deserialize(deserializer)?;
        Ok(Self)
    }
}

/// Error returned by [`MsgraphSend`] and the raw `$value` coroutines.
#[derive(Debug, Error)]
pub enum MsgraphSendError {
    /// The underlying HTTP/1.1 exchange failed.
    #[error("Microsoft Graph HTTP request failed: {0}")]
    Send(#[from] Http11SendError),
    /// The JSON request body could not be serialized.
    #[error("Microsoft Graph request serialization failed: {0}")]
    SerializeRequest(#[source] serde_json::Error),
    /// The 2xx response body could not be deserialized.
    #[error("Microsoft Graph response parsing failed: {0}")]
    ParseResponse(#[source] serde_json::Error),
    /// The request URL could not be built.
    #[error("Microsoft Graph URL parsing failed: {0}")]
    ParseUrl(#[from] url::ParseError),
    /// The request arguments were rejected before sending.
    #[error("Invalid Microsoft Graph request: {0}")]
    InvalidRequest(String),
    /// The Graph API answered a non-2xx status; carries the parsed
    /// error envelope.
    #[error("Microsoft Graph API returned HTTP {status} ({code}): {message}")]
    Api {
        /// The HTTP status of the response.
        status: u16,
        /// The error code of the Graph error envelope.
        code: String,
        /// The human-readable message of the Graph error envelope.
        message: String,
    },
    /// The server answered a 3xx; redirects are never followed.
    #[error("Microsoft Graph server returned an unexpected redirect")]
    UnexpectedRedirect,
}

impl MsgraphSendError {
    /// The HTTP status of an [`Api`](Self::Api) error, `None` for
    /// every other variant.
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// True for API statuses worth retrying (429 and common 5xx).
    pub fn is_retryable(&self) -> bool {
        matches!(self.status(), Some(429 | 500 | 502 | 503 | 504))
    }
}

/// Terminal value of every coroutine: the parsed response plus the
/// connection reuse hint.
#[derive(Clone, Debug)]
pub struct MsgraphSendOutput<T> {
    /// The parsed response body.
    pub response: T,
    /// Whether the server allows reusing the TCP/TLS connection.
    pub keep_alive: bool,
}

/// I/O-free coroutine sending one authorized Microsoft Graph request
/// and parsing its JSON response into `T`.
pub struct MsgraphSend<T> {
    state: State,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> MsgraphSend<T> {
    /// Builds a GET send for the given URL.
    pub fn get(auth: &HttpAuthBearer, url: Url) -> Self {
        Self::with_method(auth, "GET", url, None, Vec::new())
    }

    /// Builds a DELETE send for the given URL.
    pub fn delete(auth: &HttpAuthBearer, url: Url) -> Self {
        Self::with_method(auth, "DELETE", url, None, Vec::new())
    }

    /// Builds a POST send with a JSON body.
    pub fn post_json<B: Serialize>(
        auth: &HttpAuthBearer,
        url: Url,
        body: &B,
    ) -> Result<Self, MsgraphSendError> {
        let body = serde_json::to_vec(body).map_err(MsgraphSendError::SerializeRequest)?;
        Ok(Self::with_method(
            auth,
            "POST",
            url,
            Some("application/json"),
            body,
        ))
    }

    /// Builds a PATCH send with a JSON body.
    pub fn patch_json<B: Serialize>(
        auth: &HttpAuthBearer,
        url: Url,
        body: &B,
    ) -> Result<Self, MsgraphSendError> {
        let body = serde_json::to_vec(body).map_err(MsgraphSendError::SerializeRequest)?;
        Ok(Self::with_method(
            auth,
            "PATCH",
            url,
            Some("application/json"),
            body,
        ))
    }

    /// Builds a POST send with a text/plain body.
    pub fn post_text(auth: &HttpAuthBearer, url: Url, body: Vec<u8>) -> Self {
        Self::with_method(auth, "POST", url, Some("text/plain"), body)
    }

    /// Builds a send with an arbitrary method, content type and body.
    pub fn with_method(
        auth: &HttpAuthBearer,
        method: &str,
        url: Url,
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> Self {
        let mut request = HttpRequest::get(url.clone())
            .header("Accept", "application/json")
            .header("Authorization", auth.to_authorization())
            .body(body);

        if let Some(content_type) = content_type {
            request = request.header("Content-Type", content_type);
        }

        request.method = method.into();

        debug!("prepare request to send");
        trace!("method: {method}");
        trace!("url: {url}");

        Self {
            state: State::Send(Http11Send::new(request)),
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned> MsgraphCoroutine for MsgraphSend<T> {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<T>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        match &mut self.state {
            State::Send(send) => match send.resume(arg) {
                HttpCoroutineState::Yielded(HttpSendYield::WantsRead) => {
                    MsgraphCoroutineState::Yielded(MsgraphYield::WantsRead)
                }
                HttpCoroutineState::Yielded(HttpSendYield::WantsWrite(bytes)) => {
                    MsgraphCoroutineState::Yielded(MsgraphYield::WantsWrite(bytes))
                }
                HttpCoroutineState::Yielded(HttpSendYield::WantsRedirect { .. }) => {
                    MsgraphCoroutineState::Complete(Err(MsgraphSendError::UnexpectedRedirect))
                }
                HttpCoroutineState::Complete(Err(err)) => {
                    MsgraphCoroutineState::Complete(Err(err.into()))
                }
                HttpCoroutineState::Complete(Ok(HttpSendOutput {
                    response,
                    keep_alive,
                    ..
                })) => {
                    if response.status.is_success() {
                        let body = if response.body.is_empty() {
                            b"null".as_slice()
                        } else {
                            response.body.as_slice()
                        };

                        match serde_json::from_slice::<T>(body) {
                            Ok(response) => {
                                MsgraphCoroutineState::Complete(Ok(MsgraphSendOutput {
                                    response,
                                    keep_alive,
                                }))
                            }
                            Err(err) => MsgraphCoroutineState::Complete(Err(
                                MsgraphSendError::ParseResponse(err),
                            )),
                        }
                    } else {
                        let (status, code, message) =
                            parse_api_error(*response.status, &response.body);
                        MsgraphCoroutineState::Complete(Err(MsgraphSendError::Api {
                            status,
                            code,
                            message,
                        }))
                    }
                }
            },
        }
    }
}

enum State {
    Send(Http11Send),
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    code: Option<String>,
    message: Option<String>,
}

/// Parse a Microsoft Graph error envelope into its
/// `(http_status, code, message)` triple.
///
/// The envelope is `{ "error": { "code", "message" } }`; when the body
/// is not that JSON shape the raw body text becomes the message, and
/// empty or missing parts fall back to `unknown` markers.
pub fn parse_api_error(http_status: u16, body: &[u8]) -> (u16, String, String) {
    if let Ok(envelope) = serde_json::from_slice::<ErrorEnvelope>(body) {
        let code = envelope
            .error
            .code
            .filter(|code| !code.trim().is_empty())
            .unwrap_or_else(|| String::from("unknown"));
        let message = envelope
            .error
            .message
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| String::from("unknown Microsoft Graph API error"));
        return (http_status, code, message);
    }

    let message = String::from_utf8_lossy(body).trim().to_string();

    if message.is_empty() {
        (
            http_status,
            String::from("unknown"),
            String::from("unknown Microsoft Graph API error"),
        )
    } else {
        (http_status, String::from("unknown"), message)
    }
}
