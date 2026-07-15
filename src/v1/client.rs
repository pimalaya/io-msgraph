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

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use std::io::{self, Read, Write};

use io_http::rfc6750::bearer::HttpAuthBearer;
#[cfg(any(
    feature = "rustls-aws",
    feature = "rustls-ring",
    feature = "native-tls"
))]
use pimalaya_stream::std::stream::StreamStd;
/// TLS backend selection re-exported from pimalaya-stream, feeding
/// [`MsgraphClientStdConnectOptions::tls`].
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
        contact_folders::{
            MsgraphContactFolder, MsgraphContactFoldersListResponse,
            child_folders::MsgraphContactChildFoldersList, create::MsgraphContactFolderCreate,
            delete::MsgraphContactFolderDelete, get::MsgraphContactFolderGet,
            list::MsgraphContactFoldersList, list::MsgraphContactFoldersListParams,
            update::MsgraphContactFolderUpdate,
        },
        contacts::{
            MsgraphContact, MsgraphContactsDeltaResponse, MsgraphContactsListResponse,
            create::MsgraphContactCreate, delete::MsgraphContactDelete,
            delta::MsgraphContactsDelta, get::MsgraphContactGet, list::MsgraphContactsList,
            list::MsgraphContactsListParams, update::MsgraphContactUpdate,
        },
        get::MsgraphUserGet,
        mail_folders::{
            MsgraphMailFolder, MsgraphMailFoldersListResponse,
            child_folders::MsgraphMailChildFoldersList, copy::MsgraphMailFolderCopy,
            create::MsgraphMailFolderCreate, delete::MsgraphMailFolderDelete,
            get::MsgraphMailFolderGet, list::MsgraphMailFoldersList,
            list::MsgraphMailFoldersListParams, r#move::MsgraphMailFolderMove,
            update::MsgraphMailFolderUpdate,
        },
        messages::{
            MsgraphMessage, MsgraphMessagesListResponse,
            attachments::{
                MsgraphAttachment, MsgraphAttachmentsListResponse, create::MsgraphAttachmentCreate,
                delete::MsgraphAttachmentDelete, get_raw::MsgraphAttachmentGetRaw,
                list::MsgraphAttachmentsList,
            },
            copy::MsgraphMessageCopy,
            create::MsgraphMessageCreate,
            create_mime::MsgraphMessageCreateMime,
            delete::MsgraphMessageDelete,
            get::MsgraphMessageGet,
            get_raw::MsgraphMessageGetRaw,
            list::MsgraphMessagesList,
            list::MsgraphMessagesListParams,
            r#move::MsgraphMessageMove,
            send::MsgraphMessageSend,
            update::MsgraphMessageUpdate,
        },
        send_mail::{MsgraphMailSend, MsgraphMailSendMime},
    },
    v1::send::{MsgraphNoResponse, MsgraphSendError, MsgraphSendOutput},
};

/// Error returned by [`MsgraphClientStd`] operations.
#[derive(Debug, Error)]
pub enum MsgraphClientStdError {
    /// A coroutine completed with an error.
    #[error(transparent)]
    Send(#[from] MsgraphSendError),
    /// Reading from or writing to the stream failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Opening the TCP/TLS connection failed.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error(transparent)]
    Tls(#[from] anyhow::Error),
    /// The API base URL has no host to connect to.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error("Microsoft Graph URL `{0}` has no host")]
    UrlMissingHost(String),
    /// The API base URL scheme is neither http nor https.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    #[error(
        "Microsoft Graph URL `{url}` has unsupported scheme `{scheme}` (expected `http` or `https`)"
    )]
    UrlUnsupportedScheme {
        /// The rejected API base URL.
        url: String,
        /// The scheme of the rejected URL.
        scheme: String,
    },
}

/// Optional settings for [`MsgraphClientStd::connect`]; every field has a
/// default (the TLS backend default, and `me` as the mailbox owner).
pub struct MsgraphClientStdConnectOptions {
    /// The TLS backend configuration used to open the connection.
    #[cfg(any(
        feature = "rustls-aws",
        feature = "rustls-ring",
        feature = "native-tls"
    ))]
    pub tls: Tls,
    /// The mailbox owner: `me`, a user id or a principal name.
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

/// Std blocking Microsoft Graph client: a stream, the bearer
/// credential and the mailbox owner, with one method per operation.
pub struct MsgraphClientStd {
    /// The stream carrying the HTTPS connection to the Graph API.
    pub stream: Box<dyn MsgraphStream>,
    /// The bearer credential added to every request.
    pub auth: HttpAuthBearer,
    /// The mailbox owner: `me`, a user id or a principal name.
    pub user_id: String,
}

impl MsgraphClientStd {
    /// Builds a client over a caller-managed stream.
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

    /// Builds a client by opening a TCP/TLS connection to the Graph
    /// API endpoint through pimalaya-stream.
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
                return Err(MsgraphClientStdError::UrlUnsupportedScheme {
                    url: url.to_string(),
                    scheme: scheme.to_string(),
                });
            }
        };

        stream.set_read_timeout(Some(Duration::from_secs(30)))?;

        Ok(Self {
            stream: Box::new(stream),
            auth: HttpAuthBearer::new(token.to_string()),
            user_id,
        })
    }

    /// Replaces the underlying stream (e.g. after a connection reset).
    pub fn set_stream<S: Read + Write + Send + 'static>(&mut self, stream: S) {
        self.stream = Box::new(stream);
    }

    /// Runs the given coroutine to completion against the stream,
    /// fulfilling its read and write requests.
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

    /// Gets the profile of the mailbox owner.
    pub fn me(&mut self) -> Result<MsgraphSendOutput<MsgraphUser>, MsgraphClientStdError> {
        let coroutine = MsgraphUserGet::new(&self.auth, &self.user_id)?;
        self.run(coroutine)
    }

    /// Lists the mail folders of the mailbox.
    pub fn mail_folders_list(
        &mut self,
        params: &MsgraphMailFoldersListParams,
    ) -> Result<MsgraphSendOutput<MsgraphMailFoldersListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFoldersList::new(&self.auth, &self.user_id, params)?;
        self.run(coroutine)
    }

    /// Gets a mail folder by id.
    pub fn mail_folder_get(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Creates a mail folder.
    pub fn mail_folder_create(
        &mut self,
        folder: &MsgraphMailFolder,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderCreate::new(&self.auth, &self.user_id, folder)?;
        self.run(coroutine)
    }

    /// Updates a mail folder by id.
    pub fn mail_folder_update(
        &mut self,
        id: &str,
        folder: &MsgraphMailFolder,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderUpdate::new(&self.auth, &self.user_id, id, folder)?;
        self.run(coroutine)
    }

    /// Deletes a mail folder by id.
    pub fn mail_folder_delete(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Copies a mail folder into a destination folder.
    pub fn mail_folder_copy(
        &mut self,
        id: &str,
        destination: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderCopy::new(&self.auth, &self.user_id, id, destination)?;
        self.run(coroutine)
    }

    /// Moves a mail folder into a destination folder.
    pub fn mail_folder_move(
        &mut self,
        id: &str,
        destination: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMailFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphMailFolderMove::new(&self.auth, &self.user_id, id, destination)?;
        self.run(coroutine)
    }

    /// Lists the child folders of a mail folder.
    pub fn mail_child_folders_list(
        &mut self,
        id: &str,
        params: &MsgraphMailFoldersListParams,
    ) -> Result<MsgraphSendOutput<MsgraphMailFoldersListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMailChildFoldersList::new(&self.auth, &self.user_id, id, params)?;
        self.run(coroutine)
    }

    /// Lists the contact folders of the mailbox.
    pub fn contact_folders_list(
        &mut self,
        params: &MsgraphContactFoldersListParams,
    ) -> Result<MsgraphSendOutput<MsgraphContactFoldersListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphContactFoldersList::new(&self.auth, &self.user_id, params)?;
        self.run(coroutine)
    }

    /// Gets a contact folder by id.
    pub fn contact_folder_get(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphContactFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphContactFolderGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Creates a contact folder.
    pub fn contact_folder_create(
        &mut self,
        folder: &MsgraphContactFolder,
    ) -> Result<MsgraphSendOutput<MsgraphContactFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphContactFolderCreate::new(&self.auth, &self.user_id, folder)?;
        self.run(coroutine)
    }

    /// Updates a contact folder by id.
    pub fn contact_folder_update(
        &mut self,
        id: &str,
        folder: &MsgraphContactFolder,
    ) -> Result<MsgraphSendOutput<MsgraphContactFolder>, MsgraphClientStdError> {
        let coroutine = MsgraphContactFolderUpdate::new(&self.auth, &self.user_id, id, folder)?;
        self.run(coroutine)
    }

    /// Deletes a contact folder by id.
    pub fn contact_folder_delete(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphContactFolderDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Lists the child folders of a contact folder.
    pub fn contact_child_folders_list(
        &mut self,
        id: &str,
        params: &MsgraphContactFoldersListParams,
    ) -> Result<MsgraphSendOutput<MsgraphContactFoldersListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphContactChildFoldersList::new(&self.auth, &self.user_id, id, params)?;
        self.run(coroutine)
    }

    /// Lists the contacts of the default Contacts folder, or of the
    /// given contact folder.
    pub fn contacts_list(
        &mut self,
        folder: Option<&str>,
        params: &MsgraphContactsListParams,
    ) -> Result<MsgraphSendOutput<MsgraphContactsListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphContactsList::new(&self.auth, &self.user_id, folder, params)?;
        self.run(coroutine)
    }

    /// Gets a contact by id, optionally expanding the given relations.
    pub fn contact_get(
        &mut self,
        id: &str,
        expand: Option<&str>,
    ) -> Result<MsgraphSendOutput<MsgraphContact>, MsgraphClientStdError> {
        let coroutine = MsgraphContactGet::new(&self.auth, &self.user_id, id, expand)?;
        self.run(coroutine)
    }

    /// Creates a contact in the default Contacts folder, or in the
    /// given contact folder.
    pub fn contact_create(
        &mut self,
        folder: Option<&str>,
        contact: &MsgraphContact,
    ) -> Result<MsgraphSendOutput<MsgraphContact>, MsgraphClientStdError> {
        let coroutine = MsgraphContactCreate::new(&self.auth, &self.user_id, folder, contact)?;
        self.run(coroutine)
    }

    /// Updates a contact by id.
    pub fn contact_update(
        &mut self,
        id: &str,
        contact: &MsgraphContact,
    ) -> Result<MsgraphSendOutput<MsgraphContact>, MsgraphClientStdError> {
        let coroutine = MsgraphContactUpdate::new(&self.auth, &self.user_id, id, contact)?;
        self.run(coroutine)
    }

    /// Deletes a contact by id.
    pub fn contact_delete(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphContactDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Starts a contacts delta round over the default Contacts folder,
    /// or over the given contact folder.
    pub fn contacts_delta(
        &mut self,
        folder: Option<&str>,
        select: Option<&str>,
    ) -> Result<MsgraphSendOutput<MsgraphContactsDeltaResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphContactsDelta::new(&self.auth, &self.user_id, folder, select)?;
        self.run(coroutine)
    }

    /// Lists the messages of the whole mailbox, or of the given mail
    /// folder.
    pub fn messages_list(
        &mut self,
        folder: Option<&str>,
        params: &MsgraphMessagesListParams,
    ) -> Result<MsgraphSendOutput<MsgraphMessagesListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMessagesList::new(&self.auth, &self.user_id, folder, params)?;
        self.run(coroutine)
    }

    /// Gets a message by id.
    pub fn message_get(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageGet::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Gets the raw RFC 5322 MIME content of a message by id.
    pub fn message_get_raw(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<Vec<u8>>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageGetRaw::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Creates a draft message from JSON in the Drafts folder, or in
    /// the given mail folder.
    pub fn message_create(
        &mut self,
        folder: Option<&str>,
        message: &MsgraphMessage,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageCreate::new(&self.auth, &self.user_id, folder, message)?;
        self.run(coroutine)
    }

    /// Creates a draft message from raw RFC 5322 MIME bytes in the
    /// Drafts folder, or in the given mail folder.
    pub fn message_create_mime(
        &mut self,
        folder: Option<&str>,
        raw: &[u8],
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageCreateMime::new(&self.auth, &self.user_id, folder, raw)?;
        self.run(coroutine)
    }

    /// Updates a message by id.
    pub fn message_update(
        &mut self,
        id: &str,
        message: &MsgraphMessage,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageUpdate::new(&self.auth, &self.user_id, id, message)?;
        self.run(coroutine)
    }

    /// Deletes a message by id.
    pub fn message_delete(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageDelete::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Moves a message into a destination folder.
    pub fn message_move(
        &mut self,
        id: &str,
        destination: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageMove::new(&self.auth, &self.user_id, id, destination)?;
        self.run(coroutine)
    }

    /// Copies a message into a destination folder.
    pub fn message_copy(
        &mut self,
        id: &str,
        destination: &str,
    ) -> Result<MsgraphSendOutput<MsgraphMessage>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageCopy::new(&self.auth, &self.user_id, id, destination)?;
        self.run(coroutine)
    }

    /// Creates a file attachment on a message.
    pub fn attachment_create(
        &mut self,
        message_id: &str,
        name: &str,
        content: &[u8],
        content_type: Option<&str>,
    ) -> Result<MsgraphSendOutput<MsgraphAttachment>, MsgraphClientStdError> {
        let coroutine = MsgraphAttachmentCreate::new(
            &self.auth,
            &self.user_id,
            message_id,
            name,
            content,
            content_type,
        )?;
        self.run(coroutine)
    }

    /// Lists the attachments of a message.
    pub fn attachments_list(
        &mut self,
        message_id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphAttachmentsListResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphAttachmentsList::new(&self.auth, &self.user_id, message_id)?;
        self.run(coroutine)
    }

    /// Gets the raw content of an attachment.
    pub fn attachment_get_raw(
        &mut self,
        message_id: &str,
        attachment_id: &str,
    ) -> Result<MsgraphSendOutput<Vec<u8>>, MsgraphClientStdError> {
        let coroutine =
            MsgraphAttachmentGetRaw::new(&self.auth, &self.user_id, message_id, attachment_id)?;
        self.run(coroutine)
    }

    /// Deletes an attachment of a message.
    pub fn attachment_delete(
        &mut self,
        message_id: &str,
        attachment_id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine =
            MsgraphAttachmentDelete::new(&self.auth, &self.user_id, message_id, attachment_id)?;
        self.run(coroutine)
    }

    /// Sends an existing draft message by id.
    pub fn message_send(
        &mut self,
        id: &str,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMessageSend::new(&self.auth, &self.user_id, id)?;
        self.run(coroutine)
    }

    /// Sends a message described as JSON through the sendMail action.
    pub fn mail_send(
        &mut self,
        message: &MsgraphMessage,
        save_to_sent_items: bool,
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine =
            MsgraphMailSend::new(&self.auth, &self.user_id, message, save_to_sent_items)?;
        self.run(coroutine)
    }

    /// Sends a message given as raw RFC 5322 MIME bytes through the
    /// sendMail action.
    pub fn mail_send_mime(
        &mut self,
        raw: &[u8],
    ) -> Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphClientStdError> {
        let coroutine = MsgraphMailSendMime::new(&self.auth, &self.user_id, raw)?;
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

/// Blocking stream the client runs over, downcastable through `Any`
/// (e.g. to recover a concrete TLS stream).
pub trait MsgraphStream: Read + Write + Send + Any {
    /// The stream as a mutable `Any`, ready for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Read + Write + Send + Any> MsgraphStream for T {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
