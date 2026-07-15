//! Microsoft Graph contact folders list page.
//!
//! One page of the contact folders collection, paged through the OData
//! next link.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::contact_folders::types::contact_folder::MsgraphContactFolder;

/// One page of contact folders (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphContactFoldersListResponse {
    /// The contact folders of the page.
    #[serde(default)]
    pub value: Vec<MsgraphContactFolder>,
    /// The URL of the next page, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
