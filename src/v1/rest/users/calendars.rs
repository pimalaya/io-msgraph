//! Microsoft Graph calendars (`users.calendars`): list, get, create,
//! update, delete.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/calendar>

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::{
    field::MsgraphField,
    rest::users::{events::MsgraphOnlineMeetingProviderType, messages::MsgraphEmailAddress},
};

#[cfg(feature = "client")]
pub mod create;
#[cfg(feature = "client")]
pub mod delete;
#[cfg(feature = "client")]
pub mod get;
#[cfg(feature = "client")]
pub mod list;
#[cfg(feature = "client")]
pub mod permanent_delete;
#[cfg(feature = "client")]
pub mod update;

/// A calendar of the user. Doubles as the create/update body.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgraphCalendar {
    /// The unique identifier of the calendar.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The calendar name.
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub name: MsgraphField<String>,
    /// The theme colour (`auto`, `lightBlue`, `lightGreen`...).
    #[serde(default, skip_serializing_if = "MsgraphField::is_unset")]
    pub color: MsgraphField<String>,
    /// The colour as `#RRGGBB`, read-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hex_color: Option<String>,
    /// The version of the calendar, changing with every edit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_key: Option<String>,
    /// Whether the user can write to the calendar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_edit: Option<bool>,
    /// Whether this is the user's default calendar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_default_calendar: Option<bool>,
    /// Whether the calendar can be deleted from the user's mailbox.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_removable: Option<bool>,
    /// The owner of a shared calendar, or the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<MsgraphEmailAddress>,
    /// The providers an event of the calendar can host its online meeting
    /// with, empty when the tenant allows none, read-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_online_meeting_providers: Option<Vec<MsgraphOnlineMeetingProviderType>>,
    /// The provider an event asking for an online meeting gets, `unknown`
    /// when the tenant allows none, read-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_online_meeting_provider: Option<MsgraphOnlineMeetingProviderType>,
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use crate::v1::rest::users::{
        calendars::MsgraphCalendar, events::MsgraphOnlineMeetingProviderType,
    };

    /// A calendar of a tenant with Teams, as Graph answers it.
    const TEAMS_CALENDAR: &str = r#"{
        "id": "AAMkADczZWY5NjZhAAA=",
        "name": "Calendar",
        "isDefaultCalendar": true,
        "canEdit": true,
        "allowedOnlineMeetingProviders": ["teamsForBusiness"],
        "defaultOnlineMeetingProvider": "teamsForBusiness"
    }"#;

    /// A calendar of the test tenant, which allows no provider.
    const PLAIN_CALENDAR: &str = r#"{
        "id": "AAMkADczZWY5NjZhAAA=",
        "name": "Calendar",
        "isDefaultCalendar": true,
        "canEdit": true,
        "allowedOnlineMeetingProviders": [],
        "defaultOnlineMeetingProvider": "unknown"
    }"#;

    #[test]
    fn a_calendar_reads_its_online_meeting_providers() {
        let teams: MsgraphCalendar = serde_json::from_str(TEAMS_CALENDAR).unwrap();
        let plain: MsgraphCalendar = serde_json::from_str(PLAIN_CALENDAR).unwrap();
        assert_eq!(
            teams.allowed_online_meeting_providers,
            Some(vec![MsgraphOnlineMeetingProviderType::TeamsForBusiness])
        );
        assert_eq!(
            plain.allowed_online_meeting_providers,
            Some(Vec::new()),
            "none allowed is not none read"
        );
        assert_eq!(
            plain.default_online_meeting_provider,
            Some(MsgraphOnlineMeetingProviderType::Unknown)
        );

        let teams = serde_json::to_value(&teams).unwrap();
        let plain = serde_json::to_value(&plain).unwrap();
        assert_eq!(
            teams["allowedOnlineMeetingProviders"],
            serde_json::json!(["teamsForBusiness"])
        );
        assert_eq!(teams["defaultOnlineMeetingProvider"], "teamsForBusiness");
        assert_eq!(
            plain["allowedOnlineMeetingProviders"],
            serde_json::json!([])
        );
        assert_eq!(plain["defaultOnlineMeetingProvider"], "unknown");
    }
}
