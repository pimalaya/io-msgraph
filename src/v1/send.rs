//! HTTP/JSON transport every Microsoft Graph coroutine delegates to:
//! builds the authorized request and parses the JSON response, or the
//! Graph error envelope on failure.
//!
//! Microsoft Graph reference:
//! <https://learn.microsoft.com/en-us/graph/api/overview>.

use core::{error::Error, fmt, marker::PhantomData, time::Duration};

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

use io_http::{
    coroutine::{HttpCoroutine, HttpCoroutineState},
    rfc6750::bearer::HttpAuthBearer,
    rfc9110::{
        request::HttpRequest,
        response::HttpResponse,
        send::{HttpSendOutput, HttpSendYield},
    },
    rfc9112::send::{Http11Send, Http11SendError},
};
use log::{debug, trace};
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};
use url::Url;

use crate::coroutine::{MsgraphCoroutine, MsgraphCoroutineState, MsgraphYield};

/// Base URL of the Microsoft Graph API, version 1.0.
pub const MSGRAPH_API_BASE: &str = "https://graph.microsoft.com/v1.0/";

/// Base path segment addressing a mailbox owner: `me` as-is, any
/// other value as `users/{id}`.
///
/// Graph accepts an explicit user id or principal name under `users/`
/// but rejects `users/me`; only the bare `me` shortcut addresses the
/// authenticated user. An id [`path_segment`] refuses is refused here too.
pub fn user_path(user_id: &str) -> Result<String, MsgraphSendError> {
    if user_id == "me" {
        return Ok(String::from("me"));
    }

    Ok(format!("users/{}", path_segment(user_id)?))
}

/// Percent-encodes `raw` as one URL path segment, such as a folder or
/// message id.
///
/// Every byte outside the RFC 3986 `pchar` set is encoded, so a `/`, `?`
/// or `#` stays inside the segment rather than naming another resource.
/// An empty, `.` or `..` segment is refused: URL resolution would drop
/// it or climb a level, whether encoded or not.
pub fn path_segment(raw: &str) -> Result<String, MsgraphSendError> {
    if matches!(raw, "" | "." | "..") {
        let reason = format!("`{raw}` is not a valid path segment");
        return Err(MsgraphSendError::InvalidRequest(reason));
    }

    Ok(encode_segment(raw))
}

fn encode_segment(raw: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = String::with_capacity(raw.len());

    for &byte in raw.as_bytes() {
        let pchar = byte.is_ascii_alphanumeric() || b"-._~!$&'()*+,;=:@".contains(&byte);

        if pchar {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[usize::from(byte >> 4)] as char);
            encoded.push(HEX[usize::from(byte & 0xF)] as char);
        }
    }

    encoded
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
#[derive(Debug)]
pub enum MsgraphSendError {
    /// The underlying HTTP/1.1 exchange failed.
    Send(Http11SendError),
    /// The JSON request body could not be serialized.
    SerializeRequest(serde_json::Error),
    /// The 2xx response body could not be deserialized.
    ParseResponse(serde_json::Error),
    /// The request URL could not be built.
    ParseUrl(url::ParseError),
    /// The request arguments were rejected before sending.
    InvalidRequest(String),
    /// The Graph API answered a non-2xx status.
    Api(Box<MsgraphApiError>),
    /// The server answered a 3xx; redirects are never followed.
    UnexpectedRedirect,
}

impl fmt::Display for MsgraphSendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Send(err) => write!(f, "Microsoft Graph HTTP request failed: {err}"),
            Self::SerializeRequest(err) => {
                write!(f, "Microsoft Graph request serialization failed: {err}")
            }
            Self::ParseResponse(err) => write!(f, "Microsoft Graph response parsing failed: {err}"),
            Self::ParseUrl(err) => write!(f, "Microsoft Graph URL parsing failed: {err}"),
            Self::InvalidRequest(reason) => write!(f, "Invalid Microsoft Graph request: {reason}"),
            Self::Api(err) => err.fmt(f),
            Self::UnexpectedRedirect => {
                write!(f, "Microsoft Graph server returned an unexpected redirect")
            }
        }
    }
}

impl Error for MsgraphSendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Send(err) => Some(err),
            Self::SerializeRequest(err) | Self::ParseResponse(err) => Some(err),
            Self::ParseUrl(err) => Some(err),
            _ => None,
        }
    }
}

impl From<Http11SendError> for MsgraphSendError {
    fn from(err: Http11SendError) -> Self {
        Self::Send(err)
    }
}

impl From<MsgraphApiError> for MsgraphSendError {
    fn from(err: MsgraphApiError) -> Self {
        Self::Api(Box::new(err))
    }
}

impl From<url::ParseError> for MsgraphSendError {
    fn from(err: url::ParseError) -> Self {
        Self::ParseUrl(err)
    }
}

impl MsgraphSendError {
    /// The API error of a non-2xx answer, `None` for every other
    /// variant.
    pub fn api(&self) -> Option<&MsgraphApiError> {
        match self {
            Self::Api(err) => Some(err.as_ref()),
            _ => None,
        }
    }

    /// The HTTP status of an [`Api`](Self::Api) error, `None` for
    /// every other variant.
    pub fn status(&self) -> Option<u16> {
        self.api().map(|err| err.status)
    }

    /// How long the server asked to wait before a retry
    /// ([`MsgraphApiError::retry_after`]).
    pub fn retry_after(&self) -> Option<Duration> {
        self.api().and_then(|err| err.retry_after)
    }

    /// Whether Graph throttled the request
    /// ([`MsgraphApiError::is_throttled`]).
    pub fn is_throttled(&self) -> bool {
        self.api().is_some_and(MsgraphApiError::is_throttled)
    }

    /// Whether the request can be sent again as it is
    /// ([`MsgraphApiError::is_retryable`]).
    pub fn is_retryable(&self) -> bool {
        self.api().is_some_and(MsgraphApiError::is_retryable)
    }
}

/// A non-2xx answer of the Graph API: its error envelope, and what its
/// headers tell a caller to do next.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct MsgraphApiError {
    /// The HTTP status of the response.
    pub status: u16,
    /// The error code of the Graph error envelope.
    pub code: String,
    /// The human-readable message of the Graph error envelope.
    pub message: String,
    /// The method of the refused request, uppercase.
    pub method: String,
    /// How long the server asked to wait before a retry: the
    /// `Retry-After` header in its seconds form, a date reading as none.
    pub retry_after: Option<Duration>,
    /// The `WWW-Authenticate` header, which carries the claims challenge
    /// of a 401 the next token request has to answer.
    pub www_authenticate: Option<String>,
    /// The id Graph gave the request, from the `request-id` header, else
    /// from the envelope.
    pub request_id: Option<String>,
    /// Whether the server allows reusing the TCP/TLS connection.
    pub keep_alive: bool,
}

impl MsgraphApiError {
    /// Reads a non-2xx answer: the error envelope of its body, falling
    /// back as [`parse_api_error`] does, and its headers.
    ///
    /// Neither tells the method of the request nor the connection reuse:
    /// the exchange fills those in.
    pub fn parse<'a>(
        status: u16,
        headers: impl IntoIterator<Item = (&'a str, &'a str)>,
        body: &[u8],
    ) -> Self {
        let envelope = serde_json::from_slice::<ErrorEnvelope>(body).ok();
        let (status, code, message) = parse_api_error(status, body);

        let mut err = Self {
            status,
            code,
            message,
            request_id: envelope
                .and_then(|envelope| envelope.error.inner_error)
                .and_then(|inner| inner.request_id),
            ..Default::default()
        };

        for (name, value) in headers {
            let value = value.trim();

            if name.eq_ignore_ascii_case("retry-after") {
                err.retry_after = value.parse().ok().map(Duration::from_secs);
            } else if name.eq_ignore_ascii_case("www-authenticate") {
                err.www_authenticate = Some(value.into());
            } else if name.eq_ignore_ascii_case("request-id") {
                err.request_id = Some(value.into());
            }
        }

        err
    }

    /// The API error of the non-2xx `response` to a `method` request.
    pub(crate) fn of(method: &str, response: &HttpResponse, keep_alive: bool) -> Self {
        let headers = response.headers.iter();
        let headers = headers.map(|(name, value)| (name.as_str(), value.as_str()));

        let mut err = Self::parse(*response.status, headers, &response.body);
        err.method = method.into();
        err.keep_alive = keep_alive;

        err
    }

    /// Whether Graph throttled the request (429): it was not taken, so
    /// sending it again after [`retry_after`](Self::retry_after) is safe
    /// whatever its method.
    pub fn is_throttled(&self) -> bool {
        self.status == 429
    }

    /// Whether the request can be sent again as it is: a throttled one,
    /// or a transient 5xx on a method a repeat cannot double.
    ///
    /// A POST or a PATCH answered by a 5xx may have been applied, so it
    /// is not retryable here: a mail could be sent twice.
    pub fn is_retryable(&self) -> bool {
        let transient = matches!(self.status, 500 | 502 | 503 | 504);
        let idempotent = matches!(self.method.as_str(), "GET" | "HEAD" | "PUT" | "DELETE");

        self.is_throttled() || (transient && idempotent)
    }
}

impl fmt::Display for MsgraphApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            status,
            code,
            message,
            ..
        } = self;

        write!(
            f,
            "Microsoft Graph API returned HTTP {status} ({code}): {message}"
        )
    }
}

impl Error for MsgraphApiError {}

/// Terminal value of every coroutine: the parsed response, its headers
/// and the connection reuse hint.
#[derive(Clone, Debug)]
pub struct MsgraphSendOutput<T> {
    /// The parsed response body.
    pub response: T,
    /// The response headers, in order, names lowercased.
    pub headers: Vec<(String, String)>,
    /// Whether the server allows reusing the TCP/TLS connection.
    pub keep_alive: bool,
}

/// I/O-free coroutine sending one authorized Microsoft Graph request
/// and parsing its JSON response into `T`.
pub struct MsgraphSend<T> {
    state: State,
    method: String,
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
            state: State::Prepare(request),
            method: method.into(),
            _phantom: PhantomData,
        }
    }

    /// Adds a header to the request.
    ///
    /// Only effective before the first
    /// [`resume`](MsgraphCoroutine::resume), which serializes the
    /// request: a header added later is ignored.
    pub fn header(mut self, name: impl ToString, value: impl ToString) -> Self {
        if let State::Prepare(request) = &mut self.state {
            request.headers.push((name.to_string(), value.to_string()));
        }
        self
    }

    /// Asks Graph for at most `size` items per page of a paged
    /// collection (`Prefer: odata.maxpagesize={size}`).
    ///
    /// The preference rides on one request only: a caller following
    /// `@odata.nextLink` asks again on every page. Graph may serve
    /// fewer items than asked, and caps the size per endpoint.
    pub fn max_page_size(self, size: u32) -> Self {
        self.header("Prefer", format!("odata.maxpagesize={size}"))
    }
}

impl<T: DeserializeOwned> MsgraphCoroutine for MsgraphSend<T> {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<T>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        if let State::Prepare(request) = &mut self.state {
            let url = request.url.clone();
            let request = core::mem::replace(request, HttpRequest::get(url));
            self.state = State::Send(Http11Send::new(request));
        }

        match &mut self.state {
            State::Prepare(_) => unreachable!("the request is sent on the first resume"),
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
                            Ok(parsed) => MsgraphCoroutineState::Complete(Ok(MsgraphSendOutput {
                                response: parsed,
                                headers: response.headers,
                                keep_alive,
                            })),
                            Err(err) => MsgraphCoroutineState::Complete(Err(
                                MsgraphSendError::ParseResponse(err),
                            )),
                        }
                    } else {
                        let err = MsgraphApiError::of(&self.method, &response, keep_alive);
                        MsgraphCoroutineState::Complete(Err(err.into()))
                    }
                }
            },
        }
    }
}

enum State {
    /// The request, open to more headers until the first resume.
    Prepare(HttpRequest),
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
    #[serde(rename = "innerError", alias = "innererror")]
    inner_error: Option<InnerError>,
}

#[derive(Debug, Deserialize)]
struct InnerError {
    #[serde(rename = "request-id")]
    request_id: Option<String>,
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

#[cfg(test)]
mod tests {
    use crate::v1::send::{MsgraphSendError, path_segment, user_path};

    #[test]
    fn a_path_segment_keeps_a_graph_id_and_encodes_separators() {
        let id = "AAMkAGI2T-_x=";
        assert_eq!(path_segment(id).unwrap(), id);
        assert_eq!(path_segment("A/B?c#d e%").unwrap(), "A%2FB%3Fc%23d%20e%25");
        assert_eq!(path_segment("Envoyés").unwrap(), "Envoy%C3%A9s");
        assert_eq!(
            user_path("a b@example.org").unwrap(),
            "users/a%20b@example.org"
        );
        assert_eq!(user_path("me").unwrap(), "me");
    }

    #[test]
    fn a_path_segment_refuses_what_url_resolution_drops() {
        for raw in ["", ".", ".."] {
            let err = path_segment(raw).unwrap_err();
            assert!(matches!(err, MsgraphSendError::InvalidRequest(_)));

            let err = user_path(raw).unwrap_err();
            assert!(matches!(err, MsgraphSendError::InvalidRequest(_)));
        }
    }
}
