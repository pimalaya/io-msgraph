//! Microsoft Graph contact folder resource types.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/contactfolder>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// A contact folder in a user's mailbox. Doubles as the create/update
/// body, where only `display_name` is serialized.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactFolder {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
}

/// One page of contact folders (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphContactFoldersListResponse {
    #[serde(default)]
    pub value: Vec<MsgraphContactFolder>,
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
