//! List Microsoft Graph messages (`GET /me/messages` or
//! `GET /me/mailFolders/{id}/messages`).
//!
//! <https://learn.microsoft.com/en-us/graph/api/user-list-messages>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    msgraph_try,
    v1::{
        query::to_query_pairs,
        rest::users::messages::MsgraphMessagesListResponse,
        send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendError, MsgraphSendOutput, user_path},
    },
};

/// OData query parameters for listing messages.
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
pub struct MsgraphMessagesListParams<'a> {
    #[serde(rename = "$top")]
    pub top: Option<u32>,
    #[serde(rename = "$skip")]
    pub skip: Option<u32>,
    #[serde(rename = "$select")]
    pub select: Option<&'a str>,
    #[serde(rename = "$filter")]
    pub filter: Option<&'a str>,
    #[serde(rename = "$orderby")]
    pub orderby: Option<&'a str>,
    #[serde(rename = "$count")]
    pub count: Option<bool>,
    /// OData `$search` over the message collection (e.g. `"subject:foo"`
    /// or a bare term); Graph wraps a bare term as a free-text search.
    #[serde(rename = "$search")]
    pub search: Option<&'a str>,
}

pub struct MsgraphMessagesList {
    send: MsgraphSend<MsgraphMessagesListResponse>,
}

impl MsgraphMessagesList {
    /// Lists messages in the whole mailbox, or in `folder` when given (a
    /// folder id or a well-known name such as `inbox`).
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        folder: Option<&str>,
        params: &MsgraphMessagesListParams,
    ) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph messages listing");
        trace!("folder: {folder:?}");
        trace!("params: {params:?}");

        let user = user_path(user_id);
        let path = match folder {
            Some(folder) => format!("{user}/mailFolders/{folder}/messages"),
            None => format!("{user}/messages"),
        };
        let mut url = Url::parse(MSGRAPH_API_BASE)?.join(&path)?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = MsgraphSend::get(auth, url);

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphMessagesList {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphMessagesListResponse>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("microsoft graph messages listed");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}
