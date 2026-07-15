//! Microsoft Graph single-value extended property.
//!
//! The extended MAPI property riding along contacts, pairing a full
//! property identifier with its value.

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A single-value extended MAPI property: the full property id (the
/// `String {guid} Name <name>` form) and its value.
///
/// <https://learn.microsoft.com/en-us/graph/api/resources/singlevaluelegacyextendedproperty>
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphSingleValueExtendedProperty {
    /// The full property identifier.
    #[serde(default)]
    pub id: String,
    /// The value of the property.
    #[serde(default)]
    pub value: String,
}
