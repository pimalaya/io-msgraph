---
cairn: log
change: the-live-night-findings
landed: 2026-11-24
---

# The online meeting provider

Task M2 of the-live-night-findings. `MsgraphEvent` gains `online_meeting_provider`, a tri-state `MsgraphField<MsgraphOnlineMeetingProviderType>` (`teamsForBusiness`, `skypeForBusiness`, `skypeForConsumer`, `unknown`), and `MsgraphCalendar` gains the read-only `allowed_online_meeting_providers` (`Option<Vec<_>>`, an empty list kept apart from a member Graph did not return) and `default_online_meeting_provider`. A value this version does not know reads as `Unknown` (`#[serde(other)]`), so a provider Graph adds later never fails a calendar or event page. The iCalendar projection does not touch the field: `X-PIMDIR-ONLINE-MEETING` still sets `isOnlineMeeting` alone, a consumer picking the provider from the calendar's allowed ones.

A live read on 2026-11-24 (client credentials, `GET /users/microsoft@pimalaya.onmicrosoft.com/calendars` and one event, nothing written): the test tenant's four calendars answer `allowedOnlineMeetingProviders: []` and `defaultOnlineMeetingProvider: "unknown"`, its event `onlineMeetingProvider: "unknown"` beside `isOnlineMeeting: false` and `onlineMeeting: null`. That is why neverest's `isOnlineMeeting: true` is stored false there (neverest task N7).

Capability moved: events.md (new), "An event names its online meeting provider".

## Checked

- src/v1/rest/users/events.rs `an_event_reads_and_writes_back_its_online_meeting_provider` and src/v1/rest/users/calendars.rs `a_calendar_reads_its_online_meeting_providers`: failed before the fields (the written-back member read `Null`), pass after; `a_provider_this_version_does_not_know_reads_as_unknown` and `an_event_left_without_provider_sends_none` added with them.
