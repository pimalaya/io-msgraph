//! Microsoft Graph contacts (`users.contacts`): list, delta, get,
//! create, update, delete.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/contact>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::{field::MsgraphField, rest::users::messages::MsgraphEmailAddress};

pub mod create;
pub mod delete;
pub mod delta;
pub mod get;
pub mod list;
pub mod update;
#[cfg(feature = "vcard")]
pub mod vcard;

/// A contact in a contact folder. Doubles as the create/update body.
///
/// Unset fields are left out (an update preserves them), null fields
/// are serialized as explicit nulls (an update clears them), and a set
/// empty collection clears the collection.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphContact {
    /// The unique identifier of the contact.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The display name of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub display_name: MsgraphField<String>,
    /// The given name of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub given_name: MsgraphField<String>,
    /// The middle name of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub middle_name: MsgraphField<String>,
    /// The family name of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub surname: MsgraphField<String>,
    /// The nickname of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub nick_name: MsgraphField<String>,
    /// The title of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub title: MsgraphField<String>,
    /// The name suffix of the contact (Jr., III).
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub generation: MsgraphField<String>,
    /// How the contact is filed under in the folder listing.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub file_as: MsgraphField<String>,
    /// The email addresses of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub email_addresses: MsgraphField<Vec<MsgraphEmailAddress>>,
    /// The instant messaging addresses of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub im_addresses: MsgraphField<Vec<String>>,
    /// The business phone numbers of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub business_phones: MsgraphField<Vec<String>>,
    /// The home phone numbers of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub home_phones: MsgraphField<Vec<String>>,
    /// The mobile phone number of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub mobile_phone: MsgraphField<String>,
    /// The home address of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub home_address: MsgraphField<MsgraphPhysicalAddress>,
    /// The business address of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub business_address: MsgraphField<MsgraphPhysicalAddress>,
    /// The alternative address of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub other_address: MsgraphField<MsgraphPhysicalAddress>,
    /// The job title of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub job_title: MsgraphField<String>,
    /// The name of the contact's company.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub company_name: MsgraphField<String>,
    /// The department the contact works in.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub department: MsgraphField<String>,
    /// The office location of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub office_location: MsgraphField<String>,
    /// The profession of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub profession: MsgraphField<String>,
    /// The name of the contact's assistant.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub assistant_name: MsgraphField<String>,
    /// The name of the contact's manager.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub manager: MsgraphField<String>,
    /// The business home page of the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub business_home_page: MsgraphField<String>,
    /// Birthday as an ISO 8601 date-time (e.g. `1983-04-01T00:00:00Z`).
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub birthday: MsgraphField<String>,
    /// The name of the contact's spouse or partner.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub spouse_name: MsgraphField<String>,
    /// The names of the contact's children.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub children: MsgraphField<Vec<String>>,
    /// The free-form notes about the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub personal_notes: MsgraphField<String>,
    /// The categories associated with the contact.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub categories: MsgraphField<Vec<String>>,
    /// The identifier of the containing contact folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_folder_id: Option<String>,
    /// The creation date, as an ISO 8601 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_date_time: Option<String>,
    /// The last modification date, as an ISO 8601 date-time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date_time: Option<String>,
    /// The version marker of the contact, changing on every update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_key: Option<String>,
    /// Single-value extended MAPI properties attached to the contact.
    ///
    /// They ride inline in create and update bodies, but responses
    /// only carry them when the request `$expand`ed them by id.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub single_value_extended_properties: MsgraphField<Vec<MsgraphSingleValueExtendedProperty>>,
}

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

/// A single-value extended MAPI property: the full property id (the
/// `String {guid} Name <name>` form) and its value.
///
/// <https://learn.microsoft.com/en-us/graph/api/resources/singlevaluelegacyextendedproperty>
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MsgraphSingleValueExtendedProperty {
    /// The full property identifier.
    #[serde(default)]
    pub id: String,
    /// The value of the property.
    #[serde(default)]
    pub value: String,
}

#[cfg(any(feature = "ical", feature = "vcard"))]
impl MsgraphSingleValueExtendedProperty {
    /// The logical lines of a stash value, which holds one per line feed.
    ///
    /// A value stored before the stash held logical lines kept the folds of
    /// its source: a piece opening on a space or a tab continues the line
    /// before it, as it does in a document.
    pub(crate) fn stash_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        let mut open = false;

        for piece in self.value.split('\n') {
            let piece = piece.strip_suffix('\r').unwrap_or(piece);

            match (piece.strip_prefix([' ', '\t']), lines.last_mut()) {
                (Some(continuation), Some(line)) if open => line.push_str(continuation),
                _ => {
                    let line = piece.trim_start_matches([' ', '\t']);
                    open = !line.is_empty();
                    if open {
                        lines.push(line.into());
                    }
                }
            }
        }

        lines
    }
}

#[cfg(all(test, any(feature = "ical", feature = "vcard")))]
pub(crate) mod tests {
    use alloc::{borrow::ToOwned, string::String, vec::Vec};

    /// The first `len` characters of a value mixing separators with
    /// characters of one to four octets.
    pub(crate) fn sample(len: usize) -> String {
        "aZ 9:;,=\"\\n\u{e9}\u{20ac}\u{1d11e}-"
            .chars()
            .cycle()
            .take(len)
            .collect()
    }

    /// Folds `line` so no physical line runs over `width` octets, on a
    /// space and a tab in turn.
    pub(crate) fn folded(line: &str, width: usize, eol: &str) -> String {
        let mut out = String::new();
        let mut room = width;
        let mut tab = false;

        for (index, char) in line.char_indices() {
            if index > 0 && room < char.len_utf8() {
                out.push_str(eol);
                out.push(if tab { '\t' } else { ' ' });
                tab = !tab;
                room = width;
            }

            out.push(char);
            room = room.saturating_sub(char.len_utf8());
        }

        out
    }

    /// The logical lines of a document ended by CRLF.
    pub(crate) fn unfolded(document: &str) -> Vec<String> {
        document
            .replace("\r\n ", "")
            .replace("\r\n\t", "")
            .split("\r\n")
            .map(ToOwned::to_owned)
            .collect()
    }
}
