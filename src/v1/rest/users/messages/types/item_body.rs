//! Microsoft Graph item body.
//!
//! The body of a message: its content and the format the content is
//! written in.

use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::messages::types::body_type::MsgraphBodyType;

/// The body of a message, in text or HTML format.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphItemBody {
    /// The format of the body content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<MsgraphBodyType>,
    /// The body content itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
