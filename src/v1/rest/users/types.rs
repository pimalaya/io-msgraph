//! Microsoft Graph user resource types.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/user>

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A Microsoft Graph user (the signed-in mailbox owner via `me`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphUser {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_principal_name: Option<String>,
}
