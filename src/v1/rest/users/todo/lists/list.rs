//! List Microsoft To Do task lists (`GET /me/todo/lists`).
//!
//! <https://learn.microsoft.com/en-us/graph/api/todo-list-lists>

use alloc::format;

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::Serialize;
use url::Url;

use crate::{
    coroutine::*,
    msgraph_try,
    v1::{
        page::{MsgraphPage, link_url},
        query::to_query_pairs,
        rest::users::todo::lists::MsgraphTodoList,
        send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendError, MsgraphSendOutput, user_path},
    },
};

/// One page of task lists.
pub type MsgraphTodoListsListResponse = MsgraphPage<MsgraphTodoList>;

/// OData query parameters for listing task lists.
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
pub struct MsgraphTodoListsListParams<'a> {
    /// Maximum number of lists per page (`$top`).
    #[serde(rename = "$top")]
    pub top: Option<u32>,
    /// Number of lists to skip (`$skip`).
    #[serde(rename = "$skip")]
    pub skip: Option<u32>,
    /// Comma-separated properties to return (`$select`).
    #[serde(rename = "$select")]
    pub select: Option<&'a str>,
    /// OData filter expression (`$filter`).
    #[serde(rename = "$filter")]
    pub filter: Option<&'a str>,
}

/// Lists the Microsoft To Do task lists of the user.
pub struct MsgraphTodoListsList {
    send: MsgraphSend<MsgraphTodoListsListResponse>,
}

impl MsgraphTodoListsList {
    /// Lists the user's task lists.
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &MsgraphTodoListsListParams,
    ) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph to do lists listing");
        trace!("params: {params:?}");

        let user = user_path(user_id)?;
        let mut url = Url::parse(MSGRAPH_API_BASE)?.join(&format!("{user}/todo/lists"))?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = MsgraphSend::get(auth, url);

        Ok(Self { send })
    }

    /// Continues a listing from an `@odata.nextLink`, sent as-is. A link
    /// off the Microsoft Graph origin is refused.
    pub fn from_link(auth: &HttpAuthBearer, link: &str) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph to do lists listing from link");
        trace!("link: {link}");

        let url = link_url(link)?;
        let send = MsgraphSend::get(auth, url);

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphTodoListsList {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphTodoListsListResponse>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("to do lists listed");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}
