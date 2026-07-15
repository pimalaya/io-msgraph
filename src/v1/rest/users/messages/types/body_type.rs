//! Microsoft Graph body type.
//!
//! The format discriminator of a message body: plain text or HTML.

use serde::{Deserialize, Serialize};

/// Format of a message body.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MsgraphBodyType {
    /// Plain text body.
    Text,
    /// HTML body.
    Html,
}
