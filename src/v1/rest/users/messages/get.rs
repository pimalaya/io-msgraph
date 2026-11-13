//! Get a Microsoft Graph message (`GET /me/messages/{id}`).
//!
//! <https://learn.microsoft.com/en-us/graph/api/message-get>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use url::Url;

use crate::{
    coroutine::*,
    msgraph_try,
    v1::{
        rest::users::messages::MsgraphMessage,
        send::{
            MSGRAPH_API_BASE, MsgraphSend, MsgraphSendError, MsgraphSendOutput, path_segment,
            user_path,
        },
    },
};

/// Gets a Microsoft Graph message.
pub struct MsgraphMessageGet {
    send: MsgraphSend<MsgraphMessage>,
}

impl MsgraphMessageGet {
    /// Gets the message `id`, trimmed to the `$select`ed properties and
    /// `$expand`ing the given navigation clause when either is passed.
    ///
    /// Internet message headers come back only selected, and extended
    /// properties only expanded.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        id: &str,
        select: Option<&str>,
        expand: Option<&str>,
    ) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph message retrieval");
        trace!("id: {id:?}");
        trace!("select: {select:?}, expand: {expand:?}");

        let user = user_path(user_id)?;
        let mut url = Url::parse(MSGRAPH_API_BASE)?
            .join(&format!("{user}/messages/{}", path_segment(id)?))?;

        if let Some(select) = select {
            url.query_pairs_mut().append_pair("$select", select);
        }
        if let Some(expand) = expand {
            url.query_pairs_mut().append_pair("$expand", expand);
        }

        let send = MsgraphSend::get(auth, url);

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphMessageGet {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphMessage>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("message retrieved");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}
