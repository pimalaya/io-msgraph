//! Microsoft Graph contacts list page.
//!
//! One page of the contacts collection, paged through the OData next
//! link.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::contacts::types::contact::MsgraphContact;

/// One page of contacts (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphContactsListResponse {
    /// The contacts of the page.
    #[serde(default)]
    pub value: Vec<MsgraphContact>,
    /// The URL of the next page, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
