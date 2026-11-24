---
cairn: tasks
change: the-live-night-findings
---

# Tasks

Each task lands with a test failing first, the gate, and a spec fold.

- [x] M1: `create_from_ical` leaves an empty body out. Test: events/ical.rs `an_event_without_description_is_created_with_no_body`. Landed 2026-11-24 (cairn/log/2026-11-24-an-event-created-without-a-body.md); spec events-ical.md.
- [x] M2: `MsgraphEvent::online_meeting_provider`, `MsgraphCalendar::allowed_online_meeting_providers` and `default_online_meeting_provider` over `MsgraphOnlineMeetingProviderType`. Tests: events.rs `an_event_reads_and_writes_back_its_online_meeting_provider`, calendars.rs `a_calendar_reads_its_online_meeting_providers`. Landed 2026-11-24 (cairn/log/2026-11-24-the-online-meeting-provider.md); spec events.md. Live read 2026-11-24 (read-only): the test tenant's four calendars answer `allowedOnlineMeetingProviders: []` and `defaultOnlineMeetingProvider: "unknown"`.
