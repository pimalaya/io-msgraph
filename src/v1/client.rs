//! Std-blocking Microsoft Graph client: wraps a `Read + Write` stream
//! plus the bearer credential and runs the coroutines against
//! `graph.microsoft.com`. Gated behind the `client` feature.

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use core::time::Duration;
use core::{any::Any, fmt};

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use alloc::string::ToString;
use alloc::{boxed::Box, string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use std::io::{self, Read, Write};

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use pimalaya_stream::std::stream::StreamStd;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
pub use pimalaya_stream::tls::*;
use thiserror::Error;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use url::Url;

#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use crate::v1::send::MSGRAPH_API_BASE;
use crate::{
    coroutine::*,
    v1::rest::users::{
        MsgraphUser,
        get::MsgraphUserGet,
        mail_folders::{
            MsgraphMailFolder, MsgraphMailFoldersListResponse, create::MsgraphMailFolderCreate,
            delete::MsgraphMailFolderDelete, get::MsgraphMailFolderGet,
            list::MsgraphMailFoldersList, list::MsgraphMailFoldersListParams,
        },
        messages::{
            MsgraphMessage, MsgraphMessagesListResponse, copy::MsgraphMessageCopy,
            create::MsgraphMessageCreate, delete::MsgraphMessageDelete, get::MsgraphMessageGet,
            get_raw::MsgraphMessageGetRaw, list::MsgraphMessagesList,
            list::MsgraphMessagesListParams, move_to::MsgraphMessageMove, send::MsgraphMessageSend,
            update::MsgraphMessageUpdate,
        },
        send_mail::{MsgraphSendMail, MsgraphSendMailMime},
    },
    v1::send::{MsgraphNoResponse, MsgraphSendError, MsgraphSendOutput},
};

#[derive(Debug, Error)]
pub enum MsgraphClientStdError {
    #[error(transparent)]
    Send(#[from] MsgraphSendError),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error(transparent)]
    Tls(#[from] anyhow::Error),
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Microsoft Graph URL `{0}` has no host")]
    UrlMissingHost(String),
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Microsoft Graph URL `{0}` has unsupported scheme `{1}` (expected `http` or `https`)")]
    UrlUnsupportedScheme(String, String),
}

/// Optional settings for [`MsgraphClientStd::connect`]; every field has a
/// default (the TLS backend default, and `me` as the mailbox owner).
pub struct MsgraphClientStdConnectOptions {
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub tls: Tls,
    pub user_id: String,
}

impl Default for MsgraphClientStdConnectOptions {
    fn default() -> Self {
        Self {
            #[cfg(any(
                feature = "rustls-aws",
                feature = "rustls-ring",
                feature = "native-tls"
            ))]
            tls: Tls::default(),
            user_id: String::from("me"),
        }
    }
}

const READ_BUFFER_SIZE: usize = 16 * 1024;

pub struct MsgraphClientStd {
    pub stream: Box<dyn MsgraphStream>,
    pub auth: HttpAuthBearer,
    pub user_id: String,
}

impl MsgraphClientStd {
    pub fn new<S: Read + Write + Send + 'static>(
        stream: S,
        token: impl ToString,
        options: MsgraphClientStdConnectOptions,
    ) -> Self {
        Self {
            stream: Box::new(stream),
            auth: HttpAuthBearer::new(token.to_string()),
            user_id: options.user_id,
        }
    }

    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub fn connect(
        token: impl ToString,
        options: MsgraphClientStdConnectOptions,
    ) -> Result<Self, MsgraphClientStdError> {
        let MsgraphClientStdConnectOptions { tls, user_id } = options;

        let url = Url::parse(MSGRAPH_API_BASE).expect("Microsoft Graph API base URL is valid");
        let host = url
            .host_str()
            .ok_or_else(|| MsgraphClientStdError::UrlMissingHost(url.to_string()))?;

        let stream = match url.scheme() {
            "http" => StreamStd::connect_tcp(host, url.port().unwrap_or(80))?,
            "https" => StreamStd::connect_tls(host, url.port().unwrap_or(443), &tls)?,
            scheme => {
                return Err(MsgraphClientStdError::UrlUnsupportedScheme(
                    url.to_string(),
                    scheme.to_string(),
                ));
            }
        };

        stream.set_read_timeout(Some(Duration::from_secs(30)))?;

        Ok(Self {
            stream: Box::new(stream),
            auth: HttpAuthBearer::new(token.to_string()),
            user_id,
        })
    }

    pub fn set_stream<S: Read + Write + Send + 'static>(&mut self, stream: S) {
        self.stream = Box::new(stream);
    }

    pub fn run<C, T>(
        &mut self,
        mut coroutine: C,
    ) -> Result<MsgraphSendOutput<T>, MsgraphClientStdError>
    where
        C: MsgraphCoroutine<
                Yield = MsgraphYield,
                Return = Result<MsgraphSendOutput<T>, MsgraphSendError>,
            >,
    {
        let mut buf = [0u8; READ_BUFFER_SIZE];
        let mut arg: Option<&[u8]> = None;

        loop {
            match coroutine.resume(arg.take()) {
                MsgraphCoroutineState::Complete(Ok(out)) => return Ok(out),
                MsgraphCoroutineState::Complete(Err(err)) => return Err(err.into()),
                MsgraphCoroutineState::Yielded(MsgraphYield::WantsRead) => {
                    let n = self.stream.read(&mut buf)?;
                    arg = Some(&buf[..n]);
                }
                MsgraphCoroutineState::Yielded(MsgraphYield::WantsWrite(bytes)) => {
                    self.stream.write_all(&bytes)?;
                    arg = None;
                }
            }
        }
    }

    pub fn me(&mut self) -> Result<MsgraphSendOutput<MsgraphUser>, MsgraphClientStdError> {
        let coroutine = MsgraphUserGet::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    pub fn mail_folders_list(
        &mut self,
        params: &MsgraphMailFoldersListParams,
    ) -> Result<MsgraphSendOutput<MsgraphMailFoldersListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFoldersList::new(&self.auth, &self.user_id, params)?;
        self.run(coroutine)
    }

    pub fn mail_folder_get(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn mail_folder_create(
        &mut self,
        folder: &MsgraphMailFolder,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderCreate::new(&self.auth, &self.user_id, folder)?;
        self.run(coroutine)
    }

    pub fn mail_folder_delete(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn messages_list(
        &mut self,
        folder: Option<&str>,
        params: &MsgraphMessagesListParams,
    ) -> Result<MsgraphSendOutput<MsgraphMessagesListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMessagesList::new(&self.auth, &self.user_id, folder, params)?;
        self.run(coroutine)
    }

    pub fn message_get(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_get_raw(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<Vec<u8>>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageGetRaw::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_create(
        &mut self,
        folder: Option<&str>,
        message: &MsgraphMessage,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageCreate::new(&self.auth, &self.user_id, folder, message)?;
        self.run(coroutine)
    }

    pub fn message_update(
        &mut self,
        id: &str,
        message: &MsgraphMessage,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageUpdate::new(&self.auth, &self.user_id, id, message)?;
        self.run(coroutine)
    }

    pub fn message_delete(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn message_move(
        &mut self,
        id: &str,
        destination: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageMove::new(&self.auth, &self.user_id, id, destination)?;
        self.run(coroutine)
    }

    pub fn message_copy(
        &mut self,
        id: &str,
        destination: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageCopy::new(&self.auth, &self.user_id, id, destination)?;
        self.run(coroutine)
    }

    pub fn message_send(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageSend::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    pub fn send_mail(
        &mut self,
        message: &MsgraphMessage,
        save_to_sent_items: bool,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine =
            MsgraphSendMail::new(&self.auth, &self.user_id, message, save_to_sent_items)?;
        self.run(coroutine)
    }

    pub fn send_mail_mime(
        &mut self,
        raw: &[u8],
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphSendMailMime::new(&self.auth, &self.user_id, raw)?;
        self.run(coroutine)
    }
}

impl fmt::Debug for MsgraphClientStd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MsgraphClientStd")
            .field("auth", &self.auth)
            .field("user_id", &self.user_id)
            .finish_non_exhaustive()
    }
}

pub trait MsgraphStream: Read + Write + Send + Any {
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Read + Write + Send + Any> MsgraphStream for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
