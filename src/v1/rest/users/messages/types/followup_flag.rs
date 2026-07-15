//! Microsoft Graph follow-up flag.
//!
//! The follow-up flag attached to a message, wrapping its status.

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::messages::types::flag_status::MsgraphFlagStatus;

/// Follow-up flag set on a message.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphFollowupFlag {
    /// The status of the follow-up flag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag_status: Option<MsgraphFlagStatus>,
}
