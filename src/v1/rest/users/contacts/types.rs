//! Microsoft Graph contact resource types.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/contact>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::{rest::users::messages::MsgraphEmailAddress, types::MsgraphField};

/// A contact in a contact folder. Doubles as the create/update body:
/// unset fields are left out (an update preserves them), null fields
/// are serialized as explicit nulls (an update clears them), and a set
/// empty collection clears the collection.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContact {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub display_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub given_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub middle_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub surname: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub nick_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub title: MsgraphField<String>,
    /// How the contact is filed under in the folder listing.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub file_as: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub email_addresses: MsgraphField<Vec<MsgraphEmailAddress>>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub im_addresses: MsgraphField<Vec<String>>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub business_phones: MsgraphField<Vec<String>>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub home_phones: MsgraphField<Vec<String>>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub mobile_phone: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub home_address: MsgraphField<MsgraphPhysicalAddress>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub business_address: MsgraphField<MsgraphPhysicalAddress>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub other_address: MsgraphField<MsgraphPhysicalAddress>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub job_title: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub company_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub department: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub office_location: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub profession: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub assistant_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub manager: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub business_home_page: MsgraphField<String>,
    /// Birthday as an ISO 8601 date-time (e.g. `1983-04-01T00:00:00Z`).
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub birthday: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub spouse_name: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub children: MsgraphField<Vec<String>>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub personal_notes: MsgraphField<String>,
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub categories: MsgraphField<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_key: Option<String>,
    /// Single-value extended MAPI properties attached to the contact.
    /// They ride inline in create and update bodies, but responses only
    /// carry them when the request `$expand`ed them by id.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub single_value_extended_properties: MsgraphField<Vec<MsgraphSingleValueExtendedProperty>>,
}

/// A single-value extended MAPI property: the full property id (the
/// `String {guid} Name <name>` form) and its value.
///
/// <https://learn.microsoft.com/en-us/graph/api/resources/singlevaluelegacyextendedproperty>
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphSingleValueExtendedProperty {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub value: String,
}

/// A physical address of a contact (home, business or other); the
/// whole object is replaced at once, so its components stay plain
/// options.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphPhysicalAddress {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub country_or_region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
}

/// One page of contacts (`value` plus the OData paging link).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphContactsListResponse {
    #[serde(default)]
    pub value: Vec<MsgraphContact>,
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}
