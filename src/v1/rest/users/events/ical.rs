//! # iCalendar projection
//!
//! Projects a Microsoft Graph event onto iCalendar and back. Graph exposes
//! no iCalendar form of an event, so a consumer needing one (a calendar
//! client, a sync engine) synthesizes the document of record from the JSON
//! resource ([`MsgraphEvent::to_ical`], [`MsgraphEvent::to_ical_series`])
//! and reads it back ([`MsgraphEvent::from_ical`]).
//!
//! A field is managed only where it has a well-defined iCalendar slot:
//!
//! - the recurrence pattern maps onto an RRULE and back, a rule Graph
//!   cannot hold being refused rather than approximated; the cancelled
//!   occurrences of a series become EXDATEs, and an exception, an
//!   occurrence edited on its own, its own VEVENT with a RECURRENCE-ID;
//! - Graph names zones the Windows way when Outlook created the event, so
//!   a Windows name is read as its CLDR IANA counterpart, every TZID gets a
//!   VTIMEZONE from the bundled database, and a zone neither table knows
//!   keeps its name, undefined;
//! - `showAs` rides the `X-MICROSOFT-CDO-BUSYSTATUS` property Outlook
//!   itself writes, and the web link is minted read-only as
//!   `X-MSGRAPH-WEB-LINK`;
//! - everything else, the UID among it, Graph minting its own `iCalUId`, is
//!   stashed verbatim in a single-value extended property
//!   ([`MSGRAPH_EVENT_STASH_ID`]) and spliced back on read; Graph returns it
//!   only expanded ([`MSGRAPH_EVENT_STASH_EXPAND`]).
//!
//! Only the series master is written back: an exception edited locally, a
//! cancelled occurrence and an EXDATE do not push, Graph taking them only
//! through the occurrence itself.

mod windows_zones;

use core::fmt;

use alloc::{
    borrow::{Cow, ToOwned},
    collections::BTreeSet,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use ical::{
    param::IcalParam,
    prop::{IcalProp, IcalPropKind, IcalPropName, action::ACTION, trigger::TRIGGER},
    tree::{
        codec::Codec,
        cst::{IcalCst, IcalItem},
        error::IcalParseError,
        line::IcalLine,
        param::{
            cn::CN, cutype::CUTYPE, fmttype::FMTTYPE, partstat::PARTSTAT, role::ROLE, tzid::TZID,
            value::VALUE,
        },
    },
    tzdb,
    value::{
        IcalValue,
        datetime::IcalDateTime,
        recur::IcalRecur,
        text::{IcalText, IcalTextList},
    },
};
use jiff::{
    Span, Timestamp,
    civil::{Date, DateTime},
    tz::TimeZone,
};
use serde_json::{from_value, to_value};

use crate::v1::{
    field::MsgraphField,
    rest::users::{
        contacts::MsgraphSingleValueExtendedProperty,
        events::{
            MsgraphAttendee, MsgraphAttendeeType, MsgraphDateTimeTimeZone, MsgraphDayOfWeek,
            MsgraphEvent, MsgraphFreeBusyStatus, MsgraphLocation, MsgraphPatternedRecurrence,
            MsgraphRecurrencePattern, MsgraphRecurrencePatternType, MsgraphRecurrenceRange,
            MsgraphRecurrenceRangeType, MsgraphResponseStatus, MsgraphResponseType,
            MsgraphSensitivity, MsgraphWeekIndex,
        },
        messages::{MsgraphBodyType, MsgraphEmailAddress, MsgraphImportance, MsgraphItemBody},
    },
};

/// Extended-property id under which the iCalendar remainder of an event is
/// stashed, a fixed app GUID plus a name (the String MAPI form).
pub const MSGRAPH_EVENT_STASH_ID: &str =
    "String {c8e5e5cf-3f6c-4f0a-9d4e-52f1e7b2a9d3} Name pimalaya-ical";

/// The `$expand` clause reading the stash back with an event, which Graph
/// omits otherwise.
pub const MSGRAPH_EVENT_STASH_EXPAND: &str = "singleValueExtendedProperties($filter=id eq \
     'String {c8e5e5cf-3f6c-4f0a-9d4e-52f1e7b2a9d3} Name pimalaya-ical')";

/// The `$select` of a read the projection is fed from: every property it
/// reads, `cancelledOccurrences` among them, which Graph returns only
/// selected, and the zones an event was created in, which give back the
/// wall time Graph answers in UTC.
pub const MSGRAPH_EVENT_ICAL_SELECT: &str = "id,iCalUId,changeKey,createdDateTime,\
     lastModifiedDateTime,type,seriesMasterId,originalStart,originalStartTimeZone,\
     originalEndTimeZone,subject,body,start,end,isAllDay,location,recurrence,attendees,\
     organizer,categories,showAs,sensitivity,importance,isReminderOn,\
     reminderMinutesBeforeStart,isCancelled,webLink,cancelledOccurrences";

/// Product identifier the synthesized document carries.
const PRODID: &str = "-//Pimalaya//io-msgraph//EN";

/// Longest raw property line stashed server-side; a longer one, an inline
/// attachment essentially, stays in the local document.
const MAX_STASH_LINE: usize = 8 * 1024;

/// Properties minted read-only from Graph-scoped fields, dropped on read.
const MINTED_PROPS: &[&str] = &["X-MSGRAPH-WEB-LINK"];

/// The Outlook busy status extension, carrying `showAs` both ways.
const BUSY_STATUS: &str = "X-MICROSOFT-CDO-BUSYSTATUS";

/// The HTML alternative description Outlook and Thunderbird exchange.
const ALT_DESC: &str = "X-ALT-DESC";

impl MsgraphEvent {
    /// Projects a lone event or a series master onto a fresh VCALENDAR
    /// document.
    pub fn to_ical(&self) -> String {
        self.to_ical_series(&[])
    }

    /// Projects a series master and its exceptions onto one VCALENDAR
    /// document, the exceptions as VEVENTs carrying the master's UID and a
    /// RECURRENCE-ID.
    pub fn to_ical_series(&self, exceptions: &[&MsgraphEvent]) -> String {
        let master = localized(self);
        let master = &*master;
        let exceptions: Vec<Cow<'_, MsgraphEvent>> = exceptions
            .iter()
            .map(|exception| localized(exception))
            .collect();
        let uid = master
            .stashed_uid()
            .or_else(|| master.ical_uid.clone())
            .unwrap_or_else(|| master.id.clone());

        let mut zones = BTreeSet::new();
        let mut events = vec![vevent(master, None, &uid, &mut zones)];
        for exception in &exceptions {
            events.push(vevent(exception, Some(master), &uid, &mut zones));
        }

        let mut calendar = IcalCst::v2();
        calendar.push(IcalProp::text(IcalPropKind::ProdId, vec![], PRODID));

        let anchor = anchor(master);
        for zone in &zones {
            if let Some(vtimezone) = tzdb::vtimezone(zone, anchor) {
                calendar.push_component(vtimezone);
            }
        }
        for event in events {
            calendar.push_component(event);
        }

        String::from_utf8_lossy(&calendar.to_bytes()).into_owned()
    }

    /// Projects an iCalendar document onto an event, in full state: each
    /// managed field is Set from the series master or Null when absent,
    /// while Graph-only fields stay Unset.
    ///
    /// The exceptions of a series are not read: Graph takes them through
    /// the occurrence itself.
    pub fn from_ical(contents: &[u8]) -> Result<Self, MsgraphEventIcalError> {
        let calendar = IcalCst::parse(contents).map_err(MsgraphEventIcalError::Parse)?;
        let vevent = master_vevent(&calendar)?;

        let mut event = MsgraphEvent {
            subject: MsgraphField::Null,
            body: MsgraphField::Null,
            location: MsgraphField::Null,
            recurrence: MsgraphField::Null,
            attendees: MsgraphField::Set(Vec::new()),
            categories: MsgraphField::Set(Vec::new()),
            show_as: MsgraphField::Null,
            sensitivity: MsgraphField::Null,
            importance: MsgraphField::Null,
            is_reminder_on: MsgraphField::Set(false),
            ..Default::default()
        };

        let mut description = None;
        let mut html = None;
        let mut start = None;
        let mut end = None;
        let mut duration = None;
        let mut transparent = None;
        let mut rrule = None;
        let mut stash = Vec::new();

        for item in &vevent.items {
            match item {
                IcalItem::Prop(line) => {
                    let name = line.name.get();
                    let consumed = if MINTED_PROPS.iter().any(|m| name.eq_ignore_ascii_case(m)) {
                        true
                    } else if name.eq_ignore_ascii_case(BUSY_STATUS) {
                        event.show_as =
                            MsgraphField::set_or_null(busy_status_from_ical(&line.raw_value_str()));
                        true
                    } else if name.eq_ignore_ascii_case(ALT_DESC)
                        && line
                            .param::<FMTTYPE>()
                            .is_some_and(|kind| kind.eq_ignore_ascii_case("text/html"))
                    {
                        html = Some(text(line));
                        true
                    } else {
                        match name.parse::<IcalPropKind>() {
                            Ok(IcalPropKind::Summary) => {
                                event.subject = MsgraphField::Set(text(line));
                                true
                            }
                            Ok(IcalPropKind::Description) => {
                                description = Some(text(line));
                                true
                            }
                            Ok(IcalPropKind::Location) => {
                                event.location = MsgraphField::Set(MsgraphLocation {
                                    display_name: Some(text(line)),
                                    ..Default::default()
                                });
                                true
                            }
                            Ok(IcalPropKind::DtStart) => {
                                start = Some(boundary(line));
                                true
                            }
                            Ok(IcalPropKind::DtEnd) => {
                                end = Some(boundary(line));
                                true
                            }
                            Ok(IcalPropKind::Duration) => {
                                duration = line.raw_value_str().trim().parse::<Span>().ok();
                                duration.is_some()
                            }
                            Ok(IcalPropKind::RRule) => {
                                rrule = Some(IcalRecur::decode(&line.value).0.trim().to_owned());
                                true
                            }
                            Ok(IcalPropKind::Categories) => {
                                let list = IcalTextList::decode(&line.value);
                                if let MsgraphField::Set(categories) = &mut event.categories {
                                    categories.extend(list.0.into_iter().map(Cow::into_owned));
                                }
                                true
                            }
                            Ok(IcalPropKind::Class) => {
                                event.sensitivity = MsgraphField::set_or_null(
                                    sensitivity_from_ical(&line.raw_value_str()),
                                );
                                true
                            }
                            Ok(IcalPropKind::Priority) => {
                                event.importance = MsgraphField::set_or_null(importance_from_ical(
                                    &line.raw_value_str(),
                                ));
                                true
                            }
                            Ok(IcalPropKind::Transp) => {
                                transparent = Some(
                                    line.raw_value_str()
                                        .trim()
                                        .eq_ignore_ascii_case("TRANSPARENT"),
                                );
                                true
                            }
                            Ok(IcalPropKind::Attendee) => {
                                if let MsgraphField::Set(attendees) = &mut event.attendees {
                                    attendees.push(attendee(line));
                                }
                                true
                            }
                            // NOTE: Graph sets the organizer, the status and
                            // the stamps itself, so an incoming value is
                            // consumed rather than stashed.
                            Ok(
                                IcalPropKind::Organizer
                                | IcalPropKind::Status
                                | IcalPropKind::DtStamp
                                | IcalPropKind::Created
                                | IcalPropKind::LastModified
                                | IcalPropKind::Sequence,
                            ) => true,
                            _ => false,
                        }
                    };

                    if !consumed {
                        let raw = raw_line(line);
                        if raw.len() <= MAX_STASH_LINE {
                            stash.push(raw);
                        }
                    }
                }
                IcalItem::Component(alarm) => {
                    let reminder = reminder_minutes(alarm);
                    match (&event.reminder_minutes_before_start, reminder) {
                        (MsgraphField::Unset, Some(minutes)) => {
                            event.is_reminder_on = MsgraphField::Set(true);
                            event.reminder_minutes_before_start = MsgraphField::Set(minutes);
                        }
                        _ => stash.extend(raw_component(alarm)),
                    }
                }
                IcalItem::Opaque(bytes) => stash.push(String::from_utf8_lossy(bytes).into_owned()),
            }
        }

        let Some(Some(start)) = start else {
            return Err(MsgraphEventIcalError::NoZonedStart);
        };
        let end = match (end, duration) {
            (Some(Some(end)), _) => end,
            (_, Some(duration)) => start
                .shifted(duration)
                .ok_or(MsgraphEventIcalError::NoEnd)?,
            _ => return Err(MsgraphEventIcalError::NoEnd),
        };

        event.is_all_day = MsgraphField::Set(start.all_day);
        if let Some(rule) = &rrule {
            event.recurrence = MsgraphField::Set(recurrence(rule, &start)?);
        }
        event.start = MsgraphField::Set(start.into_graph());
        event.end = MsgraphField::Set(end.into_graph());

        event.body = match (html, description) {
            // NOTE: the text is what an editor changes, so an HTML copy
            // it no longer matches is stale and must not win.
            (Some(html), Some(content)) if !same_text(&strip_html(&html), &content) => {
                MsgraphField::Set(MsgraphItemBody {
                    content_type: Some(MsgraphBodyType::Text),
                    content: Some(content),
                })
            }
            (Some(content), _) => MsgraphField::Set(MsgraphItemBody {
                content_type: Some(MsgraphBodyType::Html),
                content: Some(content),
            }),
            (None, Some(content)) => MsgraphField::Set(MsgraphItemBody {
                content_type: Some(MsgraphBodyType::Text),
                content: Some(content),
            }),
            (None, None) => MsgraphField::Null,
        };

        if matches!(event.show_as, MsgraphField::Null) {
            event.show_as = match transparent {
                Some(true) => MsgraphField::Set(MsgraphFreeBusyStatus::Free),
                Some(false) => MsgraphField::Set(MsgraphFreeBusyStatus::Busy),
                None => MsgraphField::Null,
            };
        }

        // NOTE: the stash entry is always Set (empty when nothing is left),
        // so an update can tell a cleared stash from an unchanged one.
        event.single_value_extended_properties =
            MsgraphField::Set(vec![MsgraphSingleValueExtendedProperty {
                id: MSGRAPH_EVENT_STASH_ID.to_string(),
                value: stash.join("\n"),
            }]);

        Ok(event)
    }

    /// Projects an iCalendar document onto a create body: the full-state
    /// projection minus its nulls and empty collections, which a fresh event
    /// does not need.
    pub fn create_from_ical(contents: &[u8]) -> Result<Self, MsgraphEventIcalError> {
        let mut event = Self::from_ical(contents)?;

        if stash_lines(&event).is_empty() {
            event.single_value_extended_properties = MsgraphField::Unset;
        }

        let mut body = to_value(&event).map_err(MsgraphEventIcalError::Json)?;
        if let Some(object) = body.as_object_mut() {
            object.retain(|_, value| {
                !value.is_null() && value.as_array().is_none_or(|array| !array.is_empty())
            });
        }

        from_value(body).map_err(MsgraphEventIcalError::Json)
    }

    /// Projects an iCalendar document onto an update body: the fields that
    /// differ from the base document (the state last synced), the unchanged
    /// ones left Unset and out of the PATCH.
    pub fn update_from_ical(contents: &[u8], base: &[u8]) -> Result<Self, MsgraphEventIcalError> {
        let mut event = Self::from_ical(contents)?;
        let base = Self::from_ical(base)?;

        macro_rules! unset_unchanged {
            ($($field:ident),* $(,)?) => {$(
                if event.$field == base.$field {
                    event.$field = MsgraphField::Unset;
                }
            )*};
        }

        unset_unchanged!(
            subject,
            body,
            start,
            end,
            is_all_day,
            location,
            recurrence,
            attendees,
            categories,
            show_as,
            sensitivity,
            importance,
            is_reminder_on,
            reminder_minutes_before_start,
            single_value_extended_properties,
        );

        Ok(event)
    }

    /// The iCalendar UID the stash carries, `None` for an event no document
    /// was ever written to, Graph minting its own `iCalUId`.
    ///
    /// A sync engine checks it after a write: a server that dropped the
    /// stash would hand the event back under Graph's UID rather than the one
    /// it was written with.
    pub fn stashed_uid(&self) -> Option<String> {
        stash_lines(self).iter().find_map(|line| {
            let mut bytes = line.clone();
            bytes.push_str("\r\n");
            let (line, _) = IcalLine::take(bytes.as_bytes()).ok()?;
            line.name
                .get()
                .eq_ignore_ascii_case("UID")
                .then(|| line.raw_value_str().trim().to_string())
                .filter(|uid| !uid.is_empty())
        })
    }
}

/// An iCalendar document that cannot become a Graph event.
#[derive(Debug)]
pub enum MsgraphEventIcalError {
    /// The document does not parse.
    Parse(IcalParseError),
    /// The document holds no VEVENT.
    NoEvent,
    /// The document holds a component Graph does not model.
    NotAnEvent(String),
    /// The VEVENT starts neither on a date, in UTC nor in a named zone.
    NoZonedStart,
    /// The VEVENT has neither a DTEND nor a DURATION.
    NoEnd,
    /// The RRULE has no Graph recurrence pattern.
    UnsupportedRecurrence(String),
    /// The projected event does not reshape through JSON.
    Json(serde_json::Error),
}

impl fmt::Display for MsgraphEventIcalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "Invalid iCalendar: {err}"),
            Self::NoEvent => write!(f, "The iCalendar contents carry no VEVENT"),
            Self::NotAnEvent(name) => {
                write!(
                    f,
                    "Microsoft Graph models no {name}: its calendars hold events only"
                )
            }
            Self::NoZonedStart => write!(
                f,
                "Microsoft Graph needs a DTSTART that is a date, in UTC, or in a named zone"
            ),
            Self::NoEnd => write!(f, "The VEVENT has neither a DTEND nor a DURATION"),
            Self::UnsupportedRecurrence(rule) => {
                write!(f, "Microsoft Graph cannot hold the recurrence `{rule}`")
            }
            Self::Json(err) => write!(f, "Cannot shape the Graph event body: {err}"),
        }
    }
}

impl core::error::Error for MsgraphEventIcalError {}

/// Builds one VEVENT; `master` is the series of an exception.
fn vevent(
    event: &MsgraphEvent,
    master: Option<&MsgraphEvent>,
    uid: &str,
    zones: &mut BTreeSet<String>,
) -> IcalCst<'static> {
    let mut vevent = IcalCst::empty("VEVENT");
    vevent.push(IcalProp::text(IcalPropKind::Uid, vec![], uid.to_owned()));

    // NOTE: DTSTAMP is mandatory (RFC 5545 3.6.1) and Graph keeps no field
    // of its own for it, so the last modification stands in.
    let stamp = event
        .last_modified_date_time
        .as_deref()
        .or(event.created_date_time.as_deref())
        .and_then(utc_stamp);
    if let Some(stamp) = &stamp {
        vevent.push(stamp_prop(IcalPropKind::DtStamp, stamp.clone()));
    }

    let all_day = event.is_all_day.as_option().copied().unwrap_or(false);
    if let Some(start) = event.start.as_option() {
        vevent.push(boundary_prop(IcalPropKind::DtStart, start, all_day, zones));
    }
    if let Some(end) = event.end.as_option() {
        vevent.push(boundary_prop(IcalPropKind::DtEnd, end, all_day, zones));
    }

    if let (Some(master), Some(original)) = (master, &event.original_start)
        && let Some(prop) = recurrence_id(master, original, zones)
    {
        vevent.push(prop);
    }

    if let Some(subject) = event.subject.as_option() {
        vevent.push(IcalProp::text(
            IcalPropKind::Summary,
            vec![],
            subject.clone(),
        ));
    }

    if let Some(body) = event.body.as_option() {
        let content = body.content.clone().unwrap_or_default();
        match body.content_type {
            Some(MsgraphBodyType::Html) => {
                vevent.push(IcalProp::text(
                    IcalPropKind::Description,
                    vec![],
                    strip_html(&content),
                ));
                vevent.push(IcalProp::text(
                    ALT_DESC,
                    vec![IcalParam::FmtType("text/html".into())],
                    content,
                ));
            }
            _ if !content.is_empty() => {
                vevent.push(IcalProp::text(IcalPropKind::Description, vec![], content));
            }
            _ => {}
        }
    }

    if let Some(name) = event
        .location
        .as_option()
        .and_then(|location| location.display_name.clone())
        .filter(|name| !name.is_empty())
    {
        vevent.push(IcalProp::text(IcalPropKind::Location, vec![], name));
    }

    if event.is_cancelled == Some(true) {
        vevent.push(IcalProp::text(IcalPropKind::Status, vec![], "CANCELLED"));
    }

    if let Some(show_as) = event.show_as.as_option() {
        let transp = match show_as {
            MsgraphFreeBusyStatus::Free => "TRANSPARENT",
            _ => "OPAQUE",
        };
        vevent.push(IcalProp::text(IcalPropKind::Transp, vec![], transp));
        vevent.push(IcalProp::text(
            BUSY_STATUS,
            vec![],
            busy_status_to_ical(*show_as),
        ));
    }

    if let Some(class) = event
        .sensitivity
        .as_option()
        .map(|s| sensitivity_to_ical(*s))
    {
        vevent.push(IcalProp::text(IcalPropKind::Class, vec![], class));
    }

    if let Some(importance) = event.importance.as_option() {
        let priority = match importance {
            MsgraphImportance::High => "1",
            MsgraphImportance::Normal => "5",
            MsgraphImportance::Low => "9",
        };
        vevent.push(IcalProp {
            name: IcalPropName::Kind(IcalPropKind::Priority),
            params: Vec::new(),
            value: IcalValue::Integer(priority.into()),
        });
    }

    let categories: Vec<Cow<'static, str>> = event
        .categories
        .as_option()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|category| !category.is_empty())
        .map(|category| Cow::Owned(category.clone()))
        .collect();
    if !categories.is_empty() {
        vevent.push(IcalProp {
            name: IcalPropName::Kind(IcalPropKind::Categories),
            params: Vec::new(),
            value: IcalValue::TextList(IcalTextList(categories)),
        });
    }

    for (kind, stamp) in [
        (IcalPropKind::Created, &event.created_date_time),
        (IcalPropKind::LastModified, &event.last_modified_date_time),
    ] {
        if let Some(stamp) = stamp.as_deref().and_then(utc_stamp) {
            vevent.push(stamp_prop(kind, stamp));
        }
    }

    if let Some(organizer) = event
        .organizer
        .as_option()
        .map(|organizer| &organizer.email_address)
    {
        vevent.push(person_prop(IcalPropKind::Organizer, organizer, Vec::new()));
    }

    for attendee in event
        .attendees
        .as_option()
        .map(Vec::as_slice)
        .unwrap_or(&[])
    {
        if let Some(prop) = attendee_prop(attendee) {
            vevent.push(prop);
        }
    }

    if master.is_none()
        && let Some(recurrence) = event.recurrence.as_option()
    {
        if let Some(rule) = rrule(recurrence, event.start.as_option(), all_day) {
            vevent.push(IcalProp {
                name: IcalPropName::Kind(IcalPropKind::RRule),
                params: Vec::new(),
                value: IcalValue::Recur(rule.into()),
            });
        }
        for exdate in exdates(event, all_day, zones) {
            vevent.push(exdate);
        }
    }

    if let Some(link) = &event.web_link {
        vevent.push(IcalProp::text("X-MSGRAPH-WEB-LINK", vec![], link.clone()));
    }

    for line in stash_lines(event) {
        if is_uid_line(&line) {
            continue;
        }
        // NOTE: a line that no longer tokenises restores nothing rather
        // than corrupting the event.
        let _ = vevent.push_raw(&line);
    }

    if event.is_reminder_on.as_option() == Some(&true)
        && let Some(minutes) = event.reminder_minutes_before_start.as_option()
    {
        let mut alarm = IcalCst::empty("VALARM");
        alarm.push(IcalProp::text(IcalPropKind::Action, vec![], "DISPLAY"));
        alarm.push(IcalProp {
            name: IcalPropName::Kind(IcalPropKind::Trigger),
            params: Vec::new(),
            value: IcalValue::Duration(format!("-PT{minutes}M").into()),
        });
        alarm.push(IcalProp::text(
            IcalPropKind::Description,
            vec![],
            "Reminder",
        ));
        vevent.push_component(alarm);
    }

    vevent
}

/// The event with a UTC start and end told back in the zones it was
/// created in.
///
/// Graph answers a read in UTC unless the request names a zone, and keeps
/// the creation zones as `originalStartTimeZone` and `originalEndTimeZone`.
/// Left in UTC, a series anchored on a wall time drifts by an hour at each
/// DST change once expanded. All-day boundaries are dates, and a zone the
/// database does not know stays UTC.
fn localized(event: &MsgraphEvent) -> Cow<'_, MsgraphEvent> {
    if event.is_all_day.as_option() == Some(&true) {
        return Cow::Borrowed(event);
    }

    let start = local_boundary(
        event.start.as_option(),
        event.original_start_time_zone.as_deref(),
    );
    let end = local_boundary(
        event.end.as_option(),
        event.original_end_time_zone.as_deref(),
    );

    if start.is_none() && end.is_none() {
        return Cow::Borrowed(event);
    }

    let mut event = event.clone();
    if let Some(start) = start {
        event.start = MsgraphField::Set(start);
    }
    if let Some(end) = end {
        event.end = MsgraphField::Set(end);
    }

    Cow::Owned(event)
}

/// A UTC boundary as the wall time of `original`, `None` when it is not
/// UTC or `original` is no IANA zone.
fn local_boundary(
    boundary: Option<&MsgraphDateTimeTimeZone>,
    original: Option<&str>,
) -> Option<MsgraphDateTimeTimeZone> {
    let boundary = boundary?;
    if !matches!(zone(boundary.time_zone.as_deref()), Zone::Utc) {
        return None;
    }
    let Zone::Iana(name) = zone(original) else {
        return None;
    };

    let utc = boundary
        .date_time
        .split('.')
        .next()?
        .parse::<DateTime>()
        .ok()?;
    let local = utc
        .to_zoned(TimeZone::UTC)
        .ok()?
        .with_time_zone(TimeZone::get(&name).ok()?)
        .datetime();

    Some(MsgraphDateTimeTimeZone {
        date_time: local.strftime("%Y-%m-%dT%H:%M:%S").to_string(),
        time_zone: Some(name),
    })
}

/// How Graph names a zone, resolved for iCalendar.
enum Zone {
    /// UTC: a `Z`-suffixed time.
    Utc,
    /// An IANA zone, given a VTIMEZONE.
    Iana(String),
    /// A name neither the Windows table nor the database knows, kept.
    Unknown(String),
}

/// Resolves a Graph zone name, Windows or IANA.
fn zone(name: Option<&str>) -> Zone {
    let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) else {
        return Zone::Utc;
    };

    if name.eq_ignore_ascii_case("UTC") || name.eq_ignore_ascii_case("Etc/UTC") {
        return Zone::Utc;
    }

    if let Some(iana) = windows_zones::iana(name) {
        return match iana {
            "Etc/UTC" => Zone::Utc,
            iana => Zone::Iana(iana.to_owned()),
        };
    }

    if tzdb::is_known(name) {
        return Zone::Iana(name.to_owned());
    }

    Zone::Unknown(name.to_owned())
}

/// A DTSTART or DTEND from a Graph wall time in a zone.
fn boundary_prop(
    kind: IcalPropKind,
    boundary: &MsgraphDateTimeTimeZone,
    all_day: bool,
    zones: &mut BTreeSet<String>,
) -> IcalProp<'static> {
    let digits = ical_digits(&boundary.date_time);

    if all_day {
        return IcalProp {
            name: IcalPropName::Kind(kind),
            params: vec![IcalParam::Value("DATE".into())],
            value: IcalValue::Date(digits.get(..8).unwrap_or_default().to_owned().into()),
        };
    }

    match zone(boundary.time_zone.as_deref()) {
        Zone::Utc => stamp_prop(kind, format!("{digits}Z")),
        Zone::Iana(zone) | Zone::Unknown(zone) => {
            zones.insert(zone.clone());
            IcalProp {
                name: IcalPropName::Kind(kind),
                params: vec![IcalParam::TzId(zone.into())],
                value: IcalValue::DateTime(digits.into()),
            }
        }
    }
}

/// The RECURRENCE-ID of an exception: its original start, a UTC instant,
/// told in the zone of its series' start, as RFC 5545 3.8.4.4 wants it.
fn recurrence_id(
    master: &MsgraphEvent,
    original: &str,
    zones: &mut BTreeSet<String>,
) -> Option<IcalProp<'static>> {
    let instant = original.parse::<Timestamp>().ok()?;
    let all_day = master.is_all_day.as_option().copied().unwrap_or(false);
    let start_zone = master
        .start
        .as_option()
        .and_then(|start| start.time_zone.as_deref());

    if all_day {
        let date = instant.to_zoned(TimeZone::UTC).date();
        return Some(IcalProp {
            name: IcalPropName::Kind(IcalPropKind::RecurrenceId),
            params: vec![IcalParam::Value("DATE".into())],
            value: IcalValue::Date(date.strftime("%Y%m%d").to_string().into()),
        });
    }

    match zone(start_zone) {
        Zone::Iana(name) => {
            let local = instant.to_zoned(TimeZone::get(&name).ok()?).datetime();
            zones.insert(name.clone());
            Some(IcalProp {
                name: IcalPropName::Kind(IcalPropKind::RecurrenceId),
                params: vec![IcalParam::TzId(name.into())],
                value: IcalValue::DateTime(local.strftime("%Y%m%dT%H%M%S").to_string().into()),
            })
        }
        _ => Some(stamp_prop(
            IcalPropKind::RecurrenceId,
            instant.strftime("%Y%m%dT%H%M%SZ").to_string(),
        )),
    }
}

/// The RRULE of a Graph recurrence, `None` for a pattern missing what its
/// kind needs.
fn rrule(
    recurrence: &MsgraphPatternedRecurrence,
    start: Option<&MsgraphDateTimeTimeZone>,
    all_day: bool,
) -> Option<String> {
    let pattern = &recurrence.pattern;
    let mut parts = Vec::new();

    let days = || -> String {
        pattern
            .days_of_week
            .iter()
            .map(|day| weekday_to_ical(*day))
            .collect::<Vec<_>>()
            .join(",")
    };

    match pattern.pattern_type? {
        MsgraphRecurrencePatternType::Daily => parts.push(String::from("FREQ=DAILY")),
        MsgraphRecurrencePatternType::Weekly => {
            parts.push(String::from("FREQ=WEEKLY"));
            if !pattern.days_of_week.is_empty() {
                parts.push(format!("BYDAY={}", days()));
            }
        }
        MsgraphRecurrencePatternType::AbsoluteMonthly => {
            parts.push(String::from("FREQ=MONTHLY"));
            parts.push(format!("BYMONTHDAY={}", pattern.day_of_month?));
        }
        MsgraphRecurrencePatternType::RelativeMonthly => {
            parts.push(String::from("FREQ=MONTHLY"));
            parts.push(format!("BYDAY={}", days()));
            parts.push(format!("BYSETPOS={}", setpos(pattern.index)));
        }
        MsgraphRecurrencePatternType::AbsoluteYearly => {
            parts.push(String::from("FREQ=YEARLY"));
            parts.push(format!("BYMONTH={}", pattern.month?));
            parts.push(format!("BYMONTHDAY={}", pattern.day_of_month?));
        }
        MsgraphRecurrencePatternType::RelativeYearly => {
            parts.push(String::from("FREQ=YEARLY"));
            parts.push(format!("BYMONTH={}", pattern.month?));
            parts.push(format!("BYDAY={}", days()));
            parts.push(format!("BYSETPOS={}", setpos(pattern.index)));
        }
    }

    if let Some(interval) = pattern.interval.filter(|interval| *interval > 1) {
        parts.push(format!("INTERVAL={interval}"));
    }

    if let Some(first) = pattern.first_day_of_week
        && pattern.pattern_type == Some(MsgraphRecurrencePatternType::Weekly)
    {
        parts.push(format!("WKST={}", weekday_to_ical(first)));
    }

    let range = &recurrence.range;
    match range.range_type {
        Some(MsgraphRecurrenceRangeType::Numbered) => {
            parts.push(format!("COUNT={}", range.number_of_occurrences?));
        }
        Some(MsgraphRecurrenceRangeType::EndDate) => {
            let end = range.end_date.as_deref()?.parse::<Date>().ok()?;
            parts.push(format!("UNTIL={}", until(end, range, start, all_day)?));
        }
        Some(MsgraphRecurrenceRangeType::NoEnd) | None => {}
    }

    Some(parts.join(";"))
}

/// The UNTIL of an `endDate` range: the date itself for an all-day series,
/// else the end of that day in the recurrence zone, told in UTC as RFC 5545
/// 3.3.10 wants it beside a zoned DTSTART.
fn until(
    end: Date,
    range: &MsgraphRecurrenceRange,
    start: Option<&MsgraphDateTimeTimeZone>,
    all_day: bool,
) -> Option<String> {
    if all_day {
        return Some(end.strftime("%Y%m%d").to_string());
    }

    let name = range
        .recurrence_time_zone
        .as_deref()
        .or(start.and_then(|start| start.time_zone.as_deref()));
    let zone = match zone(name) {
        Zone::Iana(name) => TimeZone::get(&name).ok()?,
        _ => TimeZone::UTC,
    };

    let last = end.at(23, 59, 59, 0).to_zoned(zone).ok()?;
    Some(last.timestamp().strftime("%Y%m%dT%H%M%SZ").to_string())
}

/// The EXDATEs of a series' cancelled occurrences, read off their
/// occurrence ids and told at the series' start time.
fn exdates(
    master: &MsgraphEvent,
    all_day: bool,
    zones: &mut BTreeSet<String>,
) -> Vec<IcalProp<'static>> {
    let Some(start) = master.start.as_option() else {
        return Vec::new();
    };
    let time = ical_digits(&start.date_time);
    let time = time.get(8..).unwrap_or("T000000").to_owned();

    master
        .cancelled_occurrences
        .iter()
        .filter_map(|occurrence| occurrence.rsplit('.').next())
        .filter_map(|date| date.parse::<Date>().ok())
        .map(|date| {
            let day = date.strftime("%Y%m%d").to_string();
            if all_day {
                return IcalProp {
                    name: IcalPropName::Kind(IcalPropKind::ExDate),
                    params: vec![IcalParam::Value("DATE".into())],
                    value: IcalValue::Date(day.into()),
                };
            }
            match zone(start.time_zone.as_deref()) {
                Zone::Utc => stamp_prop(IcalPropKind::ExDate, format!("{day}{time}Z")),
                Zone::Iana(name) | Zone::Unknown(name) => {
                    zones.insert(name.clone());
                    IcalProp {
                        name: IcalPropName::Kind(IcalPropKind::ExDate),
                        params: vec![IcalParam::TzId(name.into())],
                        value: IcalValue::DateTime(format!("{day}{time}").into()),
                    }
                }
            }
        })
        .collect()
}

/// A parsed DTSTART or DTEND, ready for Graph.
struct Boundary {
    /// The wall time or the midnight of the date.
    local: DateTime,
    /// Whether the boundary is a date.
    all_day: bool,
    /// The zone, `UTC` for a `Z` time and for a date.
    zone: String,
}

impl Boundary {
    /// The boundary shifted by a DURATION.
    fn shifted(&self, duration: Span) -> Option<Self> {
        Some(Self {
            local: self.local.checked_add(duration).ok()?,
            all_day: self.all_day,
            zone: self.zone.clone(),
        })
    }

    /// The Graph form of the boundary.
    fn into_graph(self) -> MsgraphDateTimeTimeZone {
        MsgraphDateTimeTimeZone {
            date_time: self.local.strftime("%Y-%m-%dT%H:%M:%S").to_string(),
            time_zone: Some(self.zone),
        }
    }
}

/// Reads a DTSTART or DTEND, `None` for a floating time, which has no
/// instant Graph could store.
fn boundary(line: &IcalLine<'_>) -> Option<Boundary> {
    let value = line.raw_value_str();
    let value = value.trim();

    let is_date = line
        .param::<VALUE>()
        .is_some_and(|kind| kind.eq_ignore_ascii_case("DATE"))
        || (value.len() == 8 && value.bytes().all(|byte| byte.is_ascii_digit()));

    if is_date {
        let date = Date::strptime("%Y%m%d", value).ok()?;
        return Some(Boundary {
            local: date.at(0, 0, 0, 0),
            all_day: true,
            zone: String::from("UTC"),
        });
    }

    let (local, zone) = match value.strip_suffix('Z') {
        Some(local) => (local, String::from("UTC")),
        None => (value, line.param::<TZID>()?.into_owned()),
    };

    Some(Boundary {
        local: DateTime::strptime("%Y%m%dT%H%M%S", local).ok()?,
        all_day: false,
        zone,
    })
}

/// The Graph recurrence of an RRULE, refused when Graph has no pattern for
/// it rather than approximated.
fn recurrence(
    rule: &str,
    start: &Boundary,
) -> Result<MsgraphPatternedRecurrence, MsgraphEventIcalError> {
    let unsupported = || MsgraphEventIcalError::UnsupportedRecurrence(rule.to_owned());

    let mut freq = None;
    let mut interval = None;
    let mut count = None;
    let mut until_value = None;
    let mut by_day = Vec::new();
    let mut by_month_day = None;
    let mut by_month = None;
    let mut by_setpos = None;
    let mut wkst = None;

    for part in rule.split(';').filter(|part| !part.is_empty()) {
        let (key, value) = part.split_once('=').ok_or_else(unsupported)?;
        match key.to_ascii_uppercase().as_str() {
            "FREQ" => freq = Some(value.to_ascii_uppercase()),
            "INTERVAL" => interval = Some(value.parse::<u32>().map_err(|_| unsupported())?),
            "COUNT" => count = Some(value.parse::<u32>().map_err(|_| unsupported())?),
            "UNTIL" => until_value = Some(value.to_owned()),
            "BYDAY" => {
                for day in value.split(',') {
                    let (ordinal, name) = day.split_at(day.len().saturating_sub(2));
                    if !ordinal.is_empty() {
                        by_setpos = Some(ordinal.parse::<i32>().map_err(|_| unsupported())?);
                    }
                    by_day.push(weekday_from_ical(name).ok_or_else(unsupported)?);
                }
            }
            "BYMONTHDAY" => by_month_day = Some(value.parse::<u32>().map_err(|_| unsupported())?),
            "BYMONTH" => by_month = Some(value.parse::<u32>().map_err(|_| unsupported())?),
            "BYSETPOS" => by_setpos = Some(value.parse::<i32>().map_err(|_| unsupported())?),
            "WKST" => wkst = Some(weekday_from_ical(value).ok_or_else(unsupported)?),
            _ => return Err(unsupported()),
        }
    }

    let index = match by_setpos {
        None => None,
        Some(1) => Some(MsgraphWeekIndex::First),
        Some(2) => Some(MsgraphWeekIndex::Second),
        Some(3) => Some(MsgraphWeekIndex::Third),
        Some(4) => Some(MsgraphWeekIndex::Fourth),
        Some(-1) => Some(MsgraphWeekIndex::Last),
        Some(_) => return Err(unsupported()),
    };

    let pattern_type = match (freq.as_deref(), by_day.is_empty(), index) {
        (Some("DAILY"), true, None) => MsgraphRecurrencePatternType::Daily,
        (Some("WEEKLY"), _, None) => MsgraphRecurrencePatternType::Weekly,
        (Some("MONTHLY"), true, None) => MsgraphRecurrencePatternType::AbsoluteMonthly,
        (Some("MONTHLY"), false, Some(_)) => MsgraphRecurrencePatternType::RelativeMonthly,
        (Some("YEARLY"), true, None) => MsgraphRecurrencePatternType::AbsoluteYearly,
        (Some("YEARLY"), false, Some(_)) => MsgraphRecurrencePatternType::RelativeYearly,
        _ => return Err(unsupported()),
    };

    let date = start.local.date();
    let mut days_of_week = by_day;
    if pattern_type == MsgraphRecurrencePatternType::Weekly && days_of_week.is_empty() {
        days_of_week.push(weekday_of(date));
    }

    let (day_of_month, month) = match pattern_type {
        MsgraphRecurrencePatternType::AbsoluteMonthly => {
            (Some(by_month_day.unwrap_or(date.day() as u32)), None)
        }
        MsgraphRecurrencePatternType::AbsoluteYearly => (
            Some(by_month_day.unwrap_or(date.day() as u32)),
            Some(by_month.unwrap_or(date.month() as u32)),
        ),
        MsgraphRecurrencePatternType::RelativeYearly => {
            (None, Some(by_month.unwrap_or(date.month() as u32)))
        }
        _ => (None, None),
    };

    let zone_name = (!start.all_day).then(|| start.zone.clone());
    let range = match (count, until_value) {
        (Some(count), _) => MsgraphRecurrenceRange {
            range_type: Some(MsgraphRecurrenceRangeType::Numbered),
            number_of_occurrences: Some(count),
            ..Default::default()
        },
        (None, Some(until)) => {
            let day = until.get(..8).ok_or_else(unsupported)?;
            let end = Date::strptime("%Y%m%d", day).map_err(|_| unsupported())?;
            MsgraphRecurrenceRange {
                range_type: Some(MsgraphRecurrenceRangeType::EndDate),
                end_date: Some(end.strftime("%Y-%m-%d").to_string()),
                ..Default::default()
            }
        }
        (None, None) => MsgraphRecurrenceRange {
            range_type: Some(MsgraphRecurrenceRangeType::NoEnd),
            ..Default::default()
        },
    };

    Ok(MsgraphPatternedRecurrence {
        pattern: MsgraphRecurrencePattern {
            pattern_type: Some(pattern_type),
            interval: Some(interval.unwrap_or(1)),
            month,
            day_of_month,
            days_of_week,
            first_day_of_week: wkst,
            index,
        },
        range: MsgraphRecurrenceRange {
            start_date: Some(date.strftime("%Y-%m-%d").to_string()),
            recurrence_time_zone: zone_name,
            ..range
        },
    })
}

/// The master VEVENT of a document: the one without a RECURRENCE-ID.
fn master_vevent<'a>(calendar: &'a IcalCst<'a>) -> Result<&'a IcalCst<'a>, MsgraphEventIcalError> {
    let components = || {
        calendar.items.iter().filter_map(|item| match item {
            IcalItem::Component(component) => Some(component.as_ref()),
            _ => None,
        })
    };

    if let Some(master) = components()
        .filter(|component| component_name(component) == "VEVENT")
        .find(|vevent| !has_prop(vevent, "RECURRENCE-ID"))
    {
        return Ok(master);
    }

    match components()
        .map(component_name)
        .find(|name| name != "VTIMEZONE" && name != "VEVENT")
    {
        Some(name) => Err(MsgraphEventIcalError::NotAnEvent(name)),
        None => Err(MsgraphEventIcalError::NoEvent),
    }
}

/// The upper-cased name a component's BEGIN line gives it.
fn component_name(component: &IcalCst<'_>) -> String {
    component
        .begin
        .as_ref()
        .map(|begin| begin.raw_value_str().trim().to_uppercase())
        .unwrap_or_default()
}

/// Whether a component carries a property of that name.
fn has_prop(component: &IcalCst<'_>, name: &str) -> bool {
    component.items.iter().any(|item| match item {
        IcalItem::Prop(line) => line.name.get().eq_ignore_ascii_case(name),
        _ => false,
    })
}

/// The lead time of a DISPLAY or AUDIO alarm triggering before the start,
/// in minutes, `None` for any alarm Graph's single reminder cannot hold.
fn reminder_minutes(alarm: &IcalCst<'_>) -> Option<u32> {
    if component_name(alarm) != "VALARM" {
        return None;
    }
    let action = alarm.prop::<ACTION>()?;
    if !matches!(action.0.trim().to_uppercase().as_str(), "DISPLAY" | "AUDIO") {
        return None;
    }

    let trigger = alarm.prop::<TRIGGER>()?;
    let lead = trigger.0.trim().strip_prefix('-')?.parse::<Span>().ok()?;
    let minutes = lead.total(jiff::Unit::Minute).ok()?;

    let whole = minutes as u32;
    (minutes >= 0.0 && f64::from(whole) == minutes).then_some(whole)
}

/// An ORGANIZER or ATTENDEE value and its CN.
fn person_prop(
    kind: IcalPropKind,
    address: &MsgraphEmailAddress,
    mut params: Vec<IcalParam<'static>>,
) -> IcalProp<'static> {
    if let Some(name) = address.name.clone().filter(|name| !name.is_empty()) {
        params.insert(0, IcalParam::Cn(name.into()));
    }
    let email = address.address.clone().unwrap_or_default();

    IcalProp {
        name: IcalPropName::Kind(kind),
        params,
        value: IcalValue::CalAddress(format!("mailto:{email}").into()),
    }
}

/// An ATTENDEE from a Graph attendee, `None` without an address.
fn attendee_prop(attendee: &MsgraphAttendee) -> Option<IcalProp<'static>> {
    let address = attendee.email_address.as_ref()?;

    let mut params = Vec::new();
    match attendee.attendee_type {
        Some(MsgraphAttendeeType::Optional) => {
            params.push(IcalParam::Role("OPT-PARTICIPANT".into()))
        }
        Some(MsgraphAttendeeType::Resource) => {
            params.push(IcalParam::Role("NON-PARTICIPANT".into()));
            params.push(IcalParam::CuType("RESOURCE".into()));
        }
        _ => params.push(IcalParam::Role("REQ-PARTICIPANT".into())),
    }

    let partstat = match attendee.status.as_ref().and_then(|status| status.response) {
        Some(MsgraphResponseType::Accepted | MsgraphResponseType::Organizer) => "ACCEPTED",
        Some(MsgraphResponseType::Declined) => "DECLINED",
        Some(MsgraphResponseType::TentativelyAccepted) => "TENTATIVE",
        _ => "NEEDS-ACTION",
    };
    params.push(IcalParam::PartStat(partstat.into()));

    Some(person_prop(IcalPropKind::Attendee, address, params))
}

/// A Graph attendee from an ATTENDEE.
fn attendee(line: &IcalLine<'_>) -> MsgraphAttendee {
    let role = line.param::<ROLE>().unwrap_or_default();
    let cutype = line.param::<CUTYPE>().unwrap_or_default();

    let attendee_type =
        if cutype.eq_ignore_ascii_case("RESOURCE") || cutype.eq_ignore_ascii_case("ROOM") {
            MsgraphAttendeeType::Resource
        } else if role.eq_ignore_ascii_case("OPT-PARTICIPANT") {
            MsgraphAttendeeType::Optional
        } else {
            MsgraphAttendeeType::Required
        };

    let response =
        line.param::<PARTSTAT>()
            .and_then(|status| match status.to_ascii_uppercase().as_str() {
                "ACCEPTED" => Some(MsgraphResponseType::Accepted),
                "DECLINED" => Some(MsgraphResponseType::Declined),
                "TENTATIVE" => Some(MsgraphResponseType::TentativelyAccepted),
                "NEEDS-ACTION" => Some(MsgraphResponseType::NotResponded),
                _ => None,
            });

    let value = line.raw_value_str();
    let value = value.trim();
    let address = value
        .strip_prefix("mailto:")
        .or_else(|| value.strip_prefix("MAILTO:"))
        .unwrap_or(value)
        .to_owned();

    MsgraphAttendee {
        attendee_type: Some(attendee_type),
        status: response.map(|response| MsgraphResponseStatus {
            response: Some(response),
            time: None,
        }),
        email_address: Some(MsgraphEmailAddress {
            name: line.param::<CN>().map(Cow::into_owned),
            address: Some(address),
        }),
    }
}

/// The time-zone anchor of a series, its start date in Unix seconds.
fn anchor(event: &MsgraphEvent) -> i64 {
    event
        .start
        .as_option()
        .and_then(|start| start.date_time.get(..10))
        .and_then(|date| date.parse::<Date>().ok())
        .and_then(|date| date.to_zoned(TimeZone::UTC).ok())
        .map_or(0, |zoned| zoned.timestamp().as_second())
}

/// A Graph date-time, `YYYY-MM-DDTHH:MM:SS[.fffffff]`, as iCalendar digits
/// `YYYYMMDDTHHMMSS`.
fn ical_digits(date_time: &str) -> String {
    let base = date_time.split('.').next().unwrap_or(date_time);
    base.chars()
        .filter(|character| *character != '-' && *character != ':')
        .take(15)
        .collect()
}

/// An RFC 3339 instant as an iCalendar UTC stamp.
fn utc_stamp(instant: &str) -> Option<String> {
    let instant = instant.parse::<Timestamp>().ok()?;
    Some(instant.strftime("%Y%m%dT%H%M%SZ").to_string())
}

/// A UTC date-time property.
fn stamp_prop(kind: IcalPropKind, stamp: String) -> IcalProp<'static> {
    IcalProp {
        name: IcalPropName::Kind(kind),
        params: Vec::new(),
        value: IcalValue::DateTime(IcalDateTime(stamp.into())),
    }
}

/// The text of an HTML body without its markup, for DESCRIPTION.
fn strip_html(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for character in html.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .trim()
        .to_owned()
}

/// Whether two texts say the same once their whitespace is collapsed, so
/// line endings and indentation the HTML source carried do not count.
fn same_text(left: &str, right: &str) -> bool {
    left.split_whitespace().eq(right.split_whitespace())
}

/// The stashed lines behind the event's extended property, matched by name.
fn stash_lines(event: &MsgraphEvent) -> Vec<String> {
    event
        .single_value_extended_properties
        .as_option()
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter(|prop| prop.id.to_ascii_lowercase().contains("pimalaya-ical"))
        .flat_map(|prop| prop.value.split('\n'))
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Whether a stashed line is the UID, written by [`vevent`] itself.
fn is_uid_line(line: &str) -> bool {
    let name = line.split([';', ':']).next().unwrap_or_default();
    name.eq_ignore_ascii_case("UID")
}

/// The decoded text of a property line.
fn text(line: &IcalLine<'_>) -> String {
    IcalText::decode(&line.value).0.into_owned()
}

/// One logical line, its ending stripped, for the stash.
fn raw_line(line: &IcalLine<'_>) -> String {
    line.to_string().trim_end_matches(['\r', '\n']).to_owned()
}

/// A whole component as its raw lines, endings stripped.
fn raw_component(component: &IcalCst<'_>) -> Vec<String> {
    component
        .to_string()
        .lines()
        .map(|line| line.trim_end_matches('\r').to_owned())
        .filter(|line| !line.is_empty())
        .collect()
}

/// The BYSETPOS of a relative pattern's week index.
fn setpos(index: Option<MsgraphWeekIndex>) -> i32 {
    match index {
        Some(MsgraphWeekIndex::Second) => 2,
        Some(MsgraphWeekIndex::Third) => 3,
        Some(MsgraphWeekIndex::Fourth) => 4,
        Some(MsgraphWeekIndex::Last) => -1,
        _ => 1,
    }
}

/// The iCalendar weekday of a Graph day.
fn weekday_to_ical(day: MsgraphDayOfWeek) -> &'static str {
    match day {
        MsgraphDayOfWeek::Sunday => "SU",
        MsgraphDayOfWeek::Monday => "MO",
        MsgraphDayOfWeek::Tuesday => "TU",
        MsgraphDayOfWeek::Wednesday => "WE",
        MsgraphDayOfWeek::Thursday => "TH",
        MsgraphDayOfWeek::Friday => "FR",
        MsgraphDayOfWeek::Saturday => "SA",
    }
}

/// The Graph day of an iCalendar weekday.
fn weekday_from_ical(day: &str) -> Option<MsgraphDayOfWeek> {
    Some(match day.to_ascii_uppercase().as_str() {
        "SU" => MsgraphDayOfWeek::Sunday,
        "MO" => MsgraphDayOfWeek::Monday,
        "TU" => MsgraphDayOfWeek::Tuesday,
        "WE" => MsgraphDayOfWeek::Wednesday,
        "TH" => MsgraphDayOfWeek::Thursday,
        "FR" => MsgraphDayOfWeek::Friday,
        "SA" => MsgraphDayOfWeek::Saturday,
        _ => return None,
    })
}

/// The Graph day a date falls on.
fn weekday_of(date: Date) -> MsgraphDayOfWeek {
    use jiff::civil::Weekday;
    match date.weekday() {
        Weekday::Sunday => MsgraphDayOfWeek::Sunday,
        Weekday::Monday => MsgraphDayOfWeek::Monday,
        Weekday::Tuesday => MsgraphDayOfWeek::Tuesday,
        Weekday::Wednesday => MsgraphDayOfWeek::Wednesday,
        Weekday::Thursday => MsgraphDayOfWeek::Thursday,
        Weekday::Friday => MsgraphDayOfWeek::Friday,
        Weekday::Saturday => MsgraphDayOfWeek::Saturday,
    }
}

/// The `X-MICROSOFT-CDO-BUSYSTATUS` value of a `showAs`.
fn busy_status_to_ical(show_as: MsgraphFreeBusyStatus) -> &'static str {
    match show_as {
        MsgraphFreeBusyStatus::Free => "FREE",
        MsgraphFreeBusyStatus::Tentative => "TENTATIVE",
        MsgraphFreeBusyStatus::Busy => "BUSY",
        MsgraphFreeBusyStatus::Oof => "OOF",
        MsgraphFreeBusyStatus::WorkingElsewhere => "WORKINGELSEWHERE",
        MsgraphFreeBusyStatus::Unknown => "UNKNOWN",
    }
}

/// The `showAs` of an `X-MICROSOFT-CDO-BUSYSTATUS` value.
fn busy_status_from_ical(value: &str) -> Option<MsgraphFreeBusyStatus> {
    Some(match value.trim().to_ascii_uppercase().as_str() {
        "FREE" => MsgraphFreeBusyStatus::Free,
        "TENTATIVE" => MsgraphFreeBusyStatus::Tentative,
        "BUSY" => MsgraphFreeBusyStatus::Busy,
        "OOF" => MsgraphFreeBusyStatus::Oof,
        "WORKINGELSEWHERE" => MsgraphFreeBusyStatus::WorkingElsewhere,
        _ => return None,
    })
}

/// The CLASS of a sensitivity; `personal` has none and reads as private.
fn sensitivity_to_ical(sensitivity: MsgraphSensitivity) -> &'static str {
    match sensitivity {
        MsgraphSensitivity::Normal => "PUBLIC",
        MsgraphSensitivity::Personal | MsgraphSensitivity::Private => "PRIVATE",
        MsgraphSensitivity::Confidential => "CONFIDENTIAL",
    }
}

/// The sensitivity of a CLASS.
fn sensitivity_from_ical(value: &str) -> Option<MsgraphSensitivity> {
    Some(match value.trim().to_ascii_uppercase().as_str() {
        "PUBLIC" => MsgraphSensitivity::Normal,
        "PRIVATE" => MsgraphSensitivity::Private,
        "CONFIDENTIAL" => MsgraphSensitivity::Confidential,
        _ => return None,
    })
}

/// The importance of a PRIORITY: 1 to 4 high, 5 normal, 6 to 9 low, 0 none.
fn importance_from_ical(value: &str) -> Option<MsgraphImportance> {
    match value.trim().parse::<u8>().ok()? {
        1..=4 => Some(MsgraphImportance::High),
        5 => Some(MsgraphImportance::Normal),
        6..=9 => Some(MsgraphImportance::Low),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::rest::users::messages::MsgraphRecipient;

    /// A weekly series in a Windows-named zone, as Outlook creates one.
    fn master() -> MsgraphEvent {
        MsgraphEvent {
            id: "SM1".into(),
            ical_uid: Some("040000008200E0".into()),
            last_modified_date_time: Some("2026-08-01T10:00:00Z".into()),
            subject: MsgraphField::Set("Stand-up".into()),
            start: MsgraphField::Set(MsgraphDateTimeTimeZone {
                date_time: "2026-08-14T09:00:00.0000000".into(),
                time_zone: Some("Romance Standard Time".into()),
            }),
            end: MsgraphField::Set(MsgraphDateTimeTimeZone {
                date_time: "2026-08-14T09:15:00.0000000".into(),
                time_zone: Some("Romance Standard Time".into()),
            }),
            is_all_day: MsgraphField::Set(false),
            recurrence: MsgraphField::Set(MsgraphPatternedRecurrence {
                pattern: MsgraphRecurrencePattern {
                    pattern_type: Some(MsgraphRecurrencePatternType::Weekly),
                    interval: Some(1),
                    days_of_week: vec![MsgraphDayOfWeek::Monday, MsgraphDayOfWeek::Friday],
                    first_day_of_week: Some(MsgraphDayOfWeek::Monday),
                    ..Default::default()
                },
                range: MsgraphRecurrenceRange {
                    range_type: Some(MsgraphRecurrenceRangeType::Numbered),
                    start_date: Some("2026-08-14".into()),
                    number_of_occurrences: Some(10),
                    ..Default::default()
                },
            }),
            organizer: MsgraphField::Set(MsgraphRecipient {
                email_address: MsgraphEmailAddress {
                    name: Some("Alice".into()),
                    address: Some("alice@x.org".into()),
                },
            }),
            attendees: MsgraphField::Set(vec![MsgraphAttendee {
                attendee_type: Some(MsgraphAttendeeType::Optional),
                status: Some(MsgraphResponseStatus {
                    response: Some(MsgraphResponseType::Accepted),
                    time: None,
                }),
                email_address: Some(MsgraphEmailAddress {
                    name: Some("Bob".into()),
                    address: Some("bob@x.org".into()),
                }),
            }]),
            show_as: MsgraphField::Set(MsgraphFreeBusyStatus::Tentative),
            is_reminder_on: MsgraphField::Set(true),
            reminder_minutes_before_start: MsgraphField::Set(15),
            cancelled_occurrences: vec!["OID.SM1.2026-08-21".into()],
            ..Default::default()
        }
    }

    #[test]
    fn a_windows_zoned_series_projects_with_its_rule_and_zone() {
        let ical = master().to_ical();

        assert!(ical.contains("UID:040000008200E0\r\n"), "{ical}");
        assert!(
            ical.contains("BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("DTSTART;TZID=Europe/Paris:20260814T090000\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("DTEND;TZID=Europe/Paris:20260814T091500\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("RRULE:FREQ=WEEKLY;BYDAY=MO,FR;WKST=MO;COUNT=10\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("EXDATE;TZID=Europe/Paris:20260821T090000\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("ORGANIZER;CN=Alice:mailto:alice@x.org\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains(
                "ATTENDEE;CN=Bob;ROLE=OPT-PARTICIPANT;PARTSTAT=ACCEPTED:mailto:bob@x.org\r\n"
            ),
            "{ical}"
        );
        assert!(
            ical.contains("X-MICROSOFT-CDO-BUSYSTATUS:TENTATIVE\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("BEGIN:VALARM\r\nACTION:DISPLAY\r\nTRIGGER:-PT15M\r\n"),
            "{ical}"
        );
    }

    #[test]
    fn an_exception_carries_the_series_uid_and_its_original_start() {
        let exception = MsgraphEvent {
            id: "EX1".into(),
            ical_uid: Some("other".into()),
            event_type: Some(crate::v1::rest::users::events::MsgraphEventType::Exception),
            series_master_id: Some("SM1".into()),
            // NOTE: 07:00 UTC is 09:00 in Paris in summer.
            original_start: Some("2026-08-17T07:00:00Z".into()),
            subject: MsgraphField::Set("Stand-up, moved".into()),
            start: MsgraphField::Set(MsgraphDateTimeTimeZone {
                date_time: "2026-08-17T10:00:00.0000000".into(),
                time_zone: Some("Romance Standard Time".into()),
            }),
            ..Default::default()
        };

        let ical = master().to_ical_series(&[&exception]);

        assert_eq!(ical.matches("BEGIN:VEVENT").count(), 2, "{ical}");
        assert_eq!(ical.matches("UID:040000008200E0\r\n").count(), 2, "{ical}");
        assert!(
            ical.contains("RECURRENCE-ID;TZID=Europe/Paris:20260817T090000\r\n"),
            "{ical}"
        );
        assert_eq!(ical.matches("RRULE:FREQ=WEEKLY").count(), 1, "{ical}");
    }

    #[test]
    fn an_all_day_event_projects_as_dates_and_back() {
        let event = MsgraphEvent {
            id: "AD1".into(),
            subject: MsgraphField::Set("Holiday".into()),
            is_all_day: MsgraphField::Set(true),
            start: MsgraphField::Set(MsgraphDateTimeTimeZone {
                date_time: "2026-08-14T00:00:00.0000000".into(),
                time_zone: Some("UTC".into()),
            }),
            end: MsgraphField::Set(MsgraphDateTimeTimeZone {
                date_time: "2026-08-15T00:00:00.0000000".into(),
                time_zone: Some("UTC".into()),
            }),
            ..Default::default()
        };

        let ical = event.to_ical();
        assert!(ical.contains("DTSTART;VALUE=DATE:20260814\r\n"), "{ical}");
        assert!(ical.contains("DTEND;VALUE=DATE:20260815\r\n"), "{ical}");

        let back = MsgraphEvent::from_ical(ical.as_bytes()).unwrap();
        assert_eq!(back.is_all_day.as_option(), Some(&true));
        assert_eq!(
            back.start.as_option().map(|start| start.date_time.as_str()),
            Some("2026-08-14T00:00:00")
        );
    }

    #[test]
    fn a_series_reads_back_with_its_recurrence() {
        let event = MsgraphEvent::from_ical(master().to_ical().as_bytes()).unwrap();

        assert_eq!(
            event.subject.as_option().map(String::as_str),
            Some("Stand-up")
        );
        let start = event.start.as_option().unwrap();
        assert_eq!(start.date_time, "2026-08-14T09:00:00");
        assert_eq!(start.time_zone.as_deref(), Some("Europe/Paris"));

        let recurrence = event.recurrence.as_option().unwrap();
        assert_eq!(
            recurrence.pattern.pattern_type,
            Some(MsgraphRecurrencePatternType::Weekly)
        );
        assert_eq!(
            recurrence.pattern.days_of_week,
            [MsgraphDayOfWeek::Monday, MsgraphDayOfWeek::Friday]
        );
        assert_eq!(recurrence.range.number_of_occurrences, Some(10));
        assert_eq!(recurrence.range.start_date.as_deref(), Some("2026-08-14"));
        assert_eq!(
            event.show_as.as_option(),
            Some(&MsgraphFreeBusyStatus::Tentative)
        );
        assert_eq!(event.reminder_minutes_before_start.as_option(), Some(&15));
        assert_eq!(event.attendees.as_option().map(Vec::len), Some(1));
    }

    #[test]
    fn a_relative_monthly_rule_round_trips_through_its_index() {
        let ical = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:u1\r\n",
            "DTSTART:20260827T120000Z\r\nDTEND:20260827T130000Z\r\n",
            "RRULE:FREQ=MONTHLY;BYDAY=TH;BYSETPOS=-1\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let event = MsgraphEvent::from_ical(ical.as_bytes()).unwrap();
        let recurrence = event.recurrence.as_option().unwrap();
        assert_eq!(
            recurrence.pattern.pattern_type,
            Some(MsgraphRecurrencePatternType::RelativeMonthly)
        );
        assert_eq!(recurrence.pattern.index, Some(MsgraphWeekIndex::Last));

        let back = event.to_ical();
        assert!(
            back.contains("RRULE:FREQ=MONTHLY;BYDAY=TH;BYSETPOS=-1\r\n"),
            "{back}"
        );
    }

    #[test]
    fn an_end_date_ends_at_midnight_in_the_series_zone() {
        let mut event = master();
        if let MsgraphField::Set(recurrence) = &mut event.recurrence {
            recurrence.range = MsgraphRecurrenceRange {
                range_type: Some(MsgraphRecurrenceRangeType::EndDate),
                start_date: Some("2026-08-14".into()),
                end_date: Some("2026-09-30".into()),
                recurrence_time_zone: Some("Romance Standard Time".into()),
                ..Default::default()
            };
        }

        // NOTE: the end of 30 September in Paris, UTC+2 in summer.
        assert!(event.to_ical().contains(";UNTIL=20260930T215959Z\r\n"));
    }

    #[test]
    fn what_graph_cannot_hold_is_refused_by_name() {
        let with = |props: &str| {
            format!(
                "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:u1\r\n{props}END:VEVENT\r\nEND:VCALENDAR\r\n"
            )
        };

        let hourly =
            with("DTSTART:20260814T090000Z\r\nDTEND:20260814T100000Z\r\nRRULE:FREQ=HOURLY\r\n");
        let floating = with("DTSTART:20260814T090000\r\nDTEND:20260814T100000\r\n");
        let todo = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:t\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";

        assert!(matches!(
            MsgraphEvent::from_ical(hourly.as_bytes()),
            Err(MsgraphEventIcalError::UnsupportedRecurrence(_))
        ));
        assert!(matches!(
            MsgraphEvent::from_ical(floating.as_bytes()),
            Err(MsgraphEventIcalError::NoZonedStart)
        ));
        assert!(matches!(
            MsgraphEvent::from_ical(todo.as_bytes()),
            Err(MsgraphEventIcalError::NotAnEvent(name)) if name == "VTODO"
        ));
    }

    #[test]
    fn a_duration_gives_the_end() {
        let ical = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:u1\r\n",
            "DTSTART;TZID=Europe/Paris:20260814T090000\r\nDURATION:PT1H30M\r\n",
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let event = MsgraphEvent::from_ical(ical.as_bytes()).unwrap();
        let end = event.end.as_option().unwrap();
        assert_eq!(end.date_time, "2026-08-14T10:30:00");
        assert_eq!(end.time_zone.as_deref(), Some("Europe/Paris"));
    }

    #[test]
    fn the_uid_and_the_remainder_ride_the_stash() {
        let ical = concat!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:urn:uuid:4fbe8971\r\n",
            "DTSTART:20260814T090000Z\r\nDTEND:20260814T100000Z\r\nSUMMARY:Lunch\r\n",
            "X-CUSTOM;X-P=1:kept\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );

        let mut event = MsgraphEvent::create_from_ical(ical.as_bytes()).unwrap();
        assert_eq!(event.stashed_uid().as_deref(), Some("urn:uuid:4fbe8971"));

        // NOTE: what Graph hands back after the create: its own ids, and
        // the stash it was given.
        event.id = "EV9".into();
        event.ical_uid = Some("040000008200E0".into());
        let back = event.to_ical();

        assert!(back.contains("UID:urn:uuid:4fbe8971\r\n"), "{back}");
        assert!(!back.contains("UID:040000008200E0"), "{back}");
        assert!(back.contains("X-CUSTOM;X-P=1:kept\r\n"), "{back}");
    }

    #[test]
    fn an_update_sends_only_what_changed() {
        let base = master().to_ical();
        let edited = base.replace("SUMMARY:Stand-up", "SUMMARY:Daily");

        let patch = MsgraphEvent::update_from_ical(edited.as_bytes(), base.as_bytes()).unwrap();

        assert_eq!(patch.subject.as_option().map(String::as_str), Some("Daily"));
        assert!(patch.start.is_unset());
        assert!(patch.recurrence.is_unset());
        assert!(patch.attendees.is_unset());
        assert!(patch.single_value_extended_properties.is_unset());
    }

    /// A read Graph answered in UTC, as it does unless asked for a zone.
    fn utc_read() -> MsgraphEvent {
        let utc = |date_time: &str| {
            MsgraphField::Set(MsgraphDateTimeTimeZone {
                date_time: date_time.into(),
                time_zone: Some("UTC".into()),
            })
        };

        MsgraphEvent {
            id: "E1".into(),
            start: utc("2030-01-10T09:00:00.0000000"),
            end: utc("2030-01-10T10:00:00.0000000"),
            is_all_day: MsgraphField::Set(false),
            original_start_time_zone: Some("Romance Standard Time".into()),
            original_end_time_zone: Some("Romance Standard Time".into()),
            ..Default::default()
        }
    }

    #[test]
    fn a_utc_read_is_told_in_its_original_zone() {
        let ical = utc_read().to_ical();

        assert!(
            ical.contains("DTSTART;TZID=Europe/Paris:20300110T100000\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("DTEND;TZID=Europe/Paris:20300110T110000\r\n"),
            "{ical}"
        );
        assert!(
            ical.contains("BEGIN:VTIMEZONE\r\nTZID:Europe/Paris\r\n"),
            "{ical}"
        );
    }

    #[test]
    fn an_unknown_original_zone_stays_utc() {
        let mut event = utc_read();
        event.original_start_time_zone = Some("tzone://Microsoft/Custom".into());
        event.original_end_time_zone = Some("tzone://Microsoft/Custom".into());

        let ical = event.to_ical();

        assert!(ical.contains("DTSTART:20300110T090000Z\r\n"), "{ical}");
    }

    #[test]
    fn an_all_day_read_keeps_its_dates() {
        let mut event = utc_read();
        event.is_all_day = MsgraphField::Set(true);
        event.start = MsgraphField::Set(MsgraphDateTimeTimeZone {
            date_time: "2030-01-10T00:00:00.0000000".into(),
            time_zone: Some("UTC".into()),
        });
        event.end = MsgraphField::Set(MsgraphDateTimeTimeZone {
            date_time: "2030-01-11T00:00:00.0000000".into(),
            time_zone: Some("UTC".into()),
        });

        let ical = event.to_ical();

        assert!(ical.contains("DTSTART;VALUE=DATE:20300110\r\n"), "{ical}");
    }

    /// A read whose text body Exchange turned into HTML, as it always does.
    fn html_read() -> String {
        let mut event = utc_read();
        event.body = MsgraphField::Set(MsgraphItemBody {
            content_type: Some(MsgraphBodyType::Html),
            content: Some(
                "<html><body>\r\n<div class=\"PlainText\">old notes</div>\r\n</body></html>".into(),
            ),
        });
        event.to_ical()
    }

    #[test]
    fn an_untouched_description_keeps_the_html() {
        let read = html_read();

        let event = MsgraphEvent::from_ical(read.as_bytes()).unwrap();

        let body = event.body.as_option().unwrap();
        assert_eq!(body.content_type, Some(MsgraphBodyType::Html));
    }

    #[test]
    fn an_edited_description_beats_its_stale_html() {
        let read = html_read();
        let edited = read.replace("DESCRIPTION:old notes", "DESCRIPTION:new notes");
        assert_ne!(edited, read, "the document carries the stripped text");

        let patch = MsgraphEvent::update_from_ical(edited.as_bytes(), read.as_bytes()).unwrap();

        let body = patch.body.as_option().expect("the edit reaches the patch");
        assert_eq!(body.content_type, Some(MsgraphBodyType::Text));
        assert_eq!(body.content.as_deref(), Some("new notes"));
    }

    #[test]
    fn the_expand_clause_names_the_stash() {
        let quoted = format!("'{MSGRAPH_EVENT_STASH_ID}'");
        assert!(MSGRAPH_EVENT_STASH_EXPAND.contains(&quoted));
    }
}
