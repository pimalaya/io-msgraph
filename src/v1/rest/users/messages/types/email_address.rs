//! Microsoft Graph email address.
//!
//! The name-and-address pair carried by recipients and contact email
//! collections.

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A named email address (`recipient.emailAddress`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphEmailAddress {
    /// The display name paired with the address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The email address itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}
