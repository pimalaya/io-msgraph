//! Microsoft Graph removed marker.
//!
//! The marker flagging a delta row as a removal, with its reason.

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// The `@removed` marker of a delta row.
///
/// <https://learn.microsoft.com/en-us/graph/delta-query-overview>
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphRemoved {
    /// `deleted` for a hard delete, `changed` for an item that left
    /// the queried scope.
    #[serde(default)]
    pub reason: String,
}
