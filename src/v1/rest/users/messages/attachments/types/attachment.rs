//! Microsoft Graph attachment resource.
//!
//! The attachment of a message, with its metadata and optional inline
//! content.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/attachment>

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A message attachment.
///
/// Graph models several subtypes (`fileAttachment`, `itemAttachment`,
/// `referenceAttachment`); this captures the common fields plus the
/// `fileAttachment` content bytes (base64) when present.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphAttachment {
    /// The unique identifier of the attachment.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The file name of the attachment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The MIME type of the attachment content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// The size of the attachment in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// Whether the attachment is displayed inline in the body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_inline: Option<bool>,
    /// The last modification date, as an ISO 8601 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date_time: Option<String>,
    /// The base64-encoded content, for the `fileAttachment` subtype.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_bytes: Option<String>,
}
