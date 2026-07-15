//! Microsoft Graph mail folders list page.
//!
//! One page of the mail folders collection, paged through the OData
//! next link.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::mail_folders::types::mail_folder::MsgraphMailFolder;

/// One page of mail folders (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphMailFoldersListResponse {
    /// The mail folders of the page.
    #[serde(default)]
    pub value: Vec<MsgraphMailFolder>,
    /// The URL of the next page, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
