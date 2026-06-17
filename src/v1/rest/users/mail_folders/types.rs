//! Microsoft Graph mail folder resource types.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/mailfolder>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// A mail folder in a user's mailbox. Doubles as the create body, where
/// only `display_name` (and optionally `is_hidden`) is serialized.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphMailFolder {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child_folder_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unread_item_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_item_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_in_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<bool>,
}

/// One page of mail folders (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphMailFoldersListResponse {
    #[serde(default)]
    pub value: Vec<MsgraphMailFolder>,
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
