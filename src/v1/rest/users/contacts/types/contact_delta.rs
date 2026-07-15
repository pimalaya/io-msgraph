//! Microsoft Graph contact delta row.
//!
//! One row of a contacts delta page: a changed contact, possibly
//! marked as removed.

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::contacts::types::{contact::MsgraphContact, removed::MsgraphRemoved};

/// One contact row of a delta page: the contact (only its id when the
/// row is a removal), plus the `@removed` marker.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphContactDelta {
    /// The changed contact.
    #[serde(flatten)]
    pub contact: MsgraphContact,
    /// The removal marker, present when the row is a removal.
    #[serde(default, rename = "@removed", skip_serializing_if = "Option::is_none")]
    pub removed: Option<MsgraphRemoved>,
}
