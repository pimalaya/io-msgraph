//! Microsoft Graph flag status.
//!
//! The status values a follow-up flag can take.

use serde::{Deserialize, Serialize};

/// Status of a follow-up flag.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MsgraphFlagStatus {
    /// The message is not flagged.
    NotFlagged,
    /// The follow-up is completed.
    Complete,
    /// The message is flagged for follow-up.
    Flagged,
}
