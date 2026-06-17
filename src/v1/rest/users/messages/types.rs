//! Microsoft Graph message resource types.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/message>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

/// A message in a mail folder. Doubles as the create/update body, where
/// only the set (non-empty) fields are serialized.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphMessage {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_preview: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<MsgraphItemBody>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<MsgraphRecipient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender: Option<MsgraphRecipient>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to_recipients: Vec<MsgraphRecipient>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc_recipients: Vec<MsgraphRecipient>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc_recipients: Vec<MsgraphRecipient>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reply_to: Vec<MsgraphRecipient>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_draft: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internet_message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub importance: Option<MsgraphImportance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag: Option<MsgraphFollowupFlag>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}

/// A named email address (`recipient.emailAddress`).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphEmailAddress {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// A message recipient, wrapping an email address.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphRecipient {
    pub email_address: MsgraphEmailAddress,
}

/// The body of a message, in text or HTML format.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphItemBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<MsgraphBodyType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Format of a message body.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MsgraphBodyType {
    Text,
    Html,
}

/// Importance of a message.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MsgraphImportance {
    Low,
    Normal,
    High,
}

/// Follow-up flag set on a message.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphFollowupFlag {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag_status: Option<MsgraphFlagStatus>,
}

/// Status of a follow-up flag.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MsgraphFlagStatus {
    NotFlagged,
    Complete,
    Flagged,
}

/// One page of messages (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphMessagesListResponse {
    #[serde(default)]
    pub value: Vec<MsgraphMessage>,
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
