//! Microsoft Graph message resource.
//!
//! The email message of a mail folder, with its headers, body,
//! recipients and flags.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/message>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::messages::types::{
    followup_flag::MsgraphFollowupFlag, importance::MsgraphImportance, item_body::MsgraphItemBody,
    recipient::MsgraphRecipient,
};

/// A message in a mail folder. Doubles as the create/update body, where
/// only the set (non-empty) fields are serialized.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphMessage {
    /// The unique identifier of the message.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The subject of the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// The first bytes of the body, in text format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_preview: Option<String>,
    /// The body of the message, in text or HTML format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<MsgraphItemBody>,
    /// The mailbox owner displayed as the author.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<MsgraphRecipient>,
    /// The account actually used to send the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender: Option<MsgraphRecipient>,
    /// The To recipients of the message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to_recipients: Vec<MsgraphRecipient>,
    /// The Cc recipients of the message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc_recipients: Vec<MsgraphRecipient>,
    /// The Bcc recipients of the message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc_recipients: Vec<MsgraphRecipient>,
    /// The addresses replies should be sent to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reply_to: Vec<MsgraphRecipient>,
    /// The reception date, as an ISO 8601 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_date_time: Option<String>,
    /// The send date, as an ISO 8601 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_date_time: Option<String>,
    /// Whether the message has been read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_read: Option<bool>,
    /// Whether the message is a draft.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_draft: Option<bool>,
    /// Whether the message carries attachments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_attachments: Option<bool>,
    /// The RFC 5322 Message-ID header of the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internet_message_id: Option<String>,
    /// The importance of the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub importance: Option<MsgraphImportance>,
    /// The follow-up flag set on the message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag: Option<MsgraphFollowupFlag>,
    /// The categories associated with the message.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// The identifier of the containing mail folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    /// The identifier of the conversation the message belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
}
