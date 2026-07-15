//! Microsoft Graph message recipient.
//!
//! The recipient wrapper used by the From, To, Cc, Bcc and Reply-To
//! collections of a message.

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::messages::types::email_address::MsgraphEmailAddress;

/// A message recipient, wrapping an email address.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphRecipient {
    /// The email address of the recipient.
    pub email_address: MsgraphEmailAddress,
}
