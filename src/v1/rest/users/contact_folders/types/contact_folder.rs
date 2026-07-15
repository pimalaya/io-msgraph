//! Microsoft Graph contact folder resource.
//!
//! A folder of contacts in a user's mailbox, with its hierarchy link.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/contactfolder>

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A contact folder in a user's mailbox. Doubles as the create/update
/// body, where only `display_name` is serialized.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContactFolder {
    /// The unique identifier of the folder.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The display name of the folder.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub display_name: String,
    /// The identifier of the parent folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
}
