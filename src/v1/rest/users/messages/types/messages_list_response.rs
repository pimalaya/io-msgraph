//! Microsoft Graph messages list page.
//!
//! One page of the messages collection, paged through the OData next
//! link.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::messages::types::message::MsgraphMessage;

/// One page of messages (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphMessagesListResponse {
    /// The messages of the page.
    #[serde(default)]
    pub value: Vec<MsgraphMessage>,
    /// The URL of the next page, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
