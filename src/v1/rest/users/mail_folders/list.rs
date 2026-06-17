//! List the Microsoft Graph mail folders (`GET /me/mailFolders`).
//!
//! <https://learn.microsoft.com/en-us/graph/api/user-list-mailfolders>

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
        rest::users::mail_folders::MsgraphMailFoldersListResponse,
        send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendError, MsgraphSendOutput, user_path},
    },
};

/// OData query parameters for listing mail folders.
#[derive(Debug, Clone, Default, Serialize, Eq, PartialEq)]
pub struct MsgraphMailFoldersListParams<'a> {
    #[serde(rename = "$top")]
    pub top: Option<u32>,
    #[serde(rename = "$skip")]
    pub skip: Option<u32>,
    #[serde(rename = "$select")]
    pub select: Option<&'a str>,
    #[serde(rename = "includeHiddenFolders")]
    pub include_hidden_folders: Option<bool>,
}

pub struct MsgraphMailFoldersList {
    send: MsgraphSend<MsgraphMailFoldersListResponse>,
}

impl MsgraphMailFoldersList {
    pub fn new(
        auth: &HttpAuthBearer,
        user_id: &str,
        params: &MsgraphMailFoldersListParams,
    ) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph mail folders listing");
        trace!("params: {params:?}");

        let user = user_path(user_id);
        let mut url = Url::parse(MSGRAPH_API_BASE)?.join(&format!("{user}/mailFolders"))?;
        url.query_pairs_mut().extend_pairs(to_query_pairs(params));

        let send = MsgraphSend::get(auth, url);

        Ok(Self { send })
    }
}

impl MsgraphCoroutine for MsgraphMailFoldersList {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphMailFoldersListResponse>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("microsoft graph mail folders listed");
        trace!("out: {out:?}");
        MsgraphCoroutineState::Complete(Ok(out))
    }
}
