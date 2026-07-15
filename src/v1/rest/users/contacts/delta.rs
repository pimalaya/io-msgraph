//! Track changes to Microsoft Graph contacts (`GET /me/contacts/delta`
//! or `GET /me/contactFolders/{id}/contacts/delta`).
//!
//! An initial request (no delta link) enumerates every contact and
//! ends with an `@odata.deltaLink`; feeding that link back through
//! [`MsgraphSend`] returns only what changed since, removals arriving
//! as `@removed`-marked rows. An expired link answers HTTP 410; the
//! consumer falls back to an initial request.
//!
//! <https://learn.microsoft.com/en-us/graph/api/contact-delta>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    msgraph_try,
    v1::{
        rest::users::contacts::MsgraphContactsDeltaResponse,
        send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendError, MsgraphSendOutput, user_path},
    },
};

/// I/O-free coroutine for the initial contacts delta request; later
/// rounds feed the returned links through [`MsgraphSend`] directly.
pub struct MsgraphContactsDelta {
    send: MsgraphSend<MsgraphContactsDeltaResponse>,
}

impl MsgraphContactsDelta {
    /// Starts a delta round over the default Contacts folder, or over
    /// `folder` when given (a contact folder id).
    ///
    /// `select` trims each row to the named properties (the id always
    /// rides along).
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        folder: Option<&str>,
        select: Option<&str>,
    ) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph contacts delta");
        trace!("folder: {folder:?}");

        let user = user_path(user_id);
        let path = match folder {
            Some(folder) => format!("{user}/contactFolders/{folder}/contacts/delta"),
            None => format!("{user}/contacts/delta"),
        };
        let mut url = Url::parse(MSGRAPH_API_BASE)?.join(&path)?;
        if let Some(select) = select {
            url.query_pairs_mut().append_pair("$select", select);
        }

        let send = MsgraphSend::get(auth, url);

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphContactsDelta {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphContactsDeltaResponse>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("contacts delta page received");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}
