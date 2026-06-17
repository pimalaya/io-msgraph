//! Microsoft Graph message attachment resource types.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/attachment>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// A message attachment. Graph models several subtypes (`fileAttachment`,
/// `itemAttachment`, `referenceAttachment`); this captures the common
/// fields plus the `fileAttachment` content bytes (base64) when present.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphAttachment {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_inline: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_bytes: Option<String>,
}

/// One page of attachments (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphAttachmentsListResponse {
    #[serde(default)]
    pub value: Vec<MsgraphAttachment>,
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
