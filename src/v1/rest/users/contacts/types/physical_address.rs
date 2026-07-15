//! Microsoft Graph physical address.
//!
//! The postal address components of a contact's home, business or
//! other address.

use alloc::string::String;

use serde::{Deserialize, Serialize};

/// A physical address of a contact (home, business or other).
///
/// The whole object is replaced at once, so its components stay plain
/// options.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphPhysicalAddress {
    /// The street name and number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    /// The city.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// The state or province.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// The country or region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_or_region: Option<String>,
    /// The postal code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}
