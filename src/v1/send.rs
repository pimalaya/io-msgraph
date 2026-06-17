//! HTTP/JSON transport every Microsoft Graph coroutine delegates to:
//! builds the authorized request and parses the JSON response, or the
//! Graph error envelope on failure.
//!
//! Microsoft Graph reference:
//! <https://learn.microsoft.com/en-us/graph/api/overview>.

use core::{fmt, marker::PhantomData};

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
use log::trace;
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use thiserror::Error;
use url::Url;

use crate::coroutine::{MsgraphCoroutine, MsgraphCoroutineState, MsgraphYield};

pub const MSGRAPH_API_BASE: &str = "https://graph.microsoft.com/v1.0/";

/// Base path segment addressing a mailbox owner: the `me` shortcut for
/// the authenticated user, or `users/{id}` for an explicit user id or
/// principal name (Graph rejects `users/me`).
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

#[derive(Debug, Error)]
pub enum MsgraphSendError {
    #[error("Microsoft Graph HTTP request failed: {0}")]
    Send(#[from] Http11SendError),
    #[error("Microsoft Graph request serialization failed: {0}")]
    SerializeRequest(#[source] serde_json::Error),
    #[error("Microsoft Graph response parsing failed: {0}")]
    ParseResponse(#[source] serde_json::Error),
    #[error("Microsoft Graph URL parsing failed: {0}")]
    ParseUrl(#[from] url::ParseError),
    #[error("Invalid Microsoft Graph request: {0}")]
    InvalidRequest(String),
    #[error("Microsoft Graph API returned HTTP {status} ({code}): {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    #[error("Microsoft Graph server returned an unexpected redirect")]
    UnexpectedRedirect,
}

impl MsgraphSendError {
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self.status(), Some(429 | 500 | 502 | 503 | 504))
    }
}

#[derive(Clone, Debug)]
pub struct MsgraphSendOutput<T> {
    pub response: T,
    pub keep_alive: bool,
}

pub struct MsgraphSend<T> {
    state: State,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> MsgraphSend<T> {
    pub fn get(auth: &HttpAuthBearer, url: Url) -> Self {
        Self::with_method(auth, "GET", url, None, Vec::new())
    }

    pub fn delete(auth: &HttpAuthBearer, url: Url) -> Self {
        Self::with_method(auth, "DELETE", url, None, Vec::new())
    }

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

    pub fn post_text(auth: &HttpAuthBearer, url: Url, body: Vec<u8>) -> Self {
        Self::with_method(auth, "POST", url, Some("text/plain"), body)
    }

    pub fn with_method(
        auth: &HttpAuthBearer,
        method: &str,
        url: Url,
        content_type: Option<&str>,
        body: Vec<u8>,
    ) -> Self {
        let host = url.host_str().unwrap_or("localhost");

        let mut request = HttpRequest::get(url.clone())
            .header("Host", host)
            .header("Accept", "application/json")
            .header("Authorization", auth.to_authorization())
            .body(body);

        if let Some(content_type) = content_type {
            request = request.header("Content-Type", content_type);
        }

        request.method = method.into();

        trace!("send Microsoft Graph {method} request to {url}");

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
        trace!("send: {}", self.state);
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

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(_) => f.write_str("send"),
        }
    }
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

/// Parse a Microsoft Graph error envelope (`{ "error": { "code",
/// "message" } }`) into `(http_status, code, message)`, falling back to
/// the raw body when it is not the expected JSON shape.
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
