//! Send a Microsoft Graph message (`POST /me/sendMail`) in JSON or MIME
//! format; the message is saved to Sent Items.
//!
//! <https://learn.microsoft.com/en-us/graph/api/user-sendmail>

use alloc::format;

use base64::{Engine, engine::general_purpose::STANDARD};
use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    msgraph_try,
    v1::{
        rest::users::messages::MsgraphMessage,
        send::{
            MSGRAPH_API_BASE, MsgraphNoResponse, MsgraphSend, MsgraphSendError, MsgraphSendOutput,
            user_path,
        },
    },
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MsgraphSendMailRequest<'a> {
    message: &'a MsgraphMessage,
    save_to_sent_items: bool,
}

/// Send a message described as a JSON [`MsgraphMessage`].
pub struct MsgraphSendMail {
    send: MsgraphSend<MsgraphNoResponse>,
}

impl MsgraphSendMail {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        message: &MsgraphMessage,
        save_to_sent_items: bool,
    ) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph mail send (json)");
        trace!("message: {message:?}");
        trace!("save_to_sent_items: {save_to_sent_items:?}");

        let url = mail_url(user_id)?;
        let body = MsgraphSendMailRequest {
            message,
            save_to_sent_items,
        };
        let send = MsgraphSend::post_json(auth, url, &body)?;

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphSendMail {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("microsoft graph mail sent (json)");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}

/// Send a message given as raw RFC 5322 MIME bytes; the MIME is
/// base64-encoded and posted as `text/plain`, as Graph requires.
pub struct MsgraphSendMailMime {
    send: MsgraphSend<MsgraphNoResponse>,
}

impl MsgraphSendMailMime {
    pub fn new(auth: &HttpAuthBearer, user_id: &str, raw: &[u8]) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph mail send (mime)");
        trace!("raw len: {}", raw.len());

        let url = mail_url(user_id)?;
        let body = STANDARD.encode(raw).into_bytes();
        let send = MsgraphSend::post_text(auth, url, body);

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphSendMailMime {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphNoResponse>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("microsoft graph mail sent (mime)");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}

fn mail_url(user_id: &str) -> Result<Url, MsgraphSendError> {
    let user = user_path(user_id);
    let url = Url::parse(MSGRAPH_API_BASE)?.join(&format!("{user}/sendMail"))?;
    Ok(url)
}
