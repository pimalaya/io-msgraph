//! Microsoft Graph attachments list page.
//!
//! One page of the attachments collection, paged through the OData
//! next link.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::messages::attachments::types::attachment::MsgraphAttachment;

/// One page of attachments (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphAttachmentsListResponse {
    /// The attachments of the page.
    #[serde(default)]
    pub value: Vec<MsgraphAttachment>,
    /// The URL of the next page, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
