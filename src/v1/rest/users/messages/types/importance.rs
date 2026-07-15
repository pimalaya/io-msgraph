//! Microsoft Graph message importance.
//!
//! The three-level importance marker of a message.

use serde::{Deserialize, Serialize};

/// Importance of a message.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MsgraphImportance {
    /// Low importance.
    Low,
    /// Normal importance, the default.
    Normal,
    /// High importance.
    High,
}
