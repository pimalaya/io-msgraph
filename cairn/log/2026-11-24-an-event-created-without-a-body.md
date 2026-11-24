---
cairn: log
change: the-live-night-findings
landed: 2026-11-24
---

# An event is created without an empty body

Task M1 of the-live-night-findings. Exchange rewrites an event 0.5 to 4 seconds after a write that sets its body (text or HTML, empty or not), moving only its `changeKey` and `lastModifiedDateTime`; a create with no body moves nothing (probed live by neverest on 2026-11-24, its cairn/log/2026-11-24-a-graph-event-created-without-a-body.md). `MsgraphEvent::create_from_ical` sent an empty text body for an event without DESCRIPTION, so every such event was rewritten after its create, and a delete queued right after met a server edit.

`create_from_ical`, and through it the master of `MsgraphEventSeries::create_from_ical`, now leaves a body with no content out. `from_ical` and `update_from_ical` are unchanged: an update still clears a removed description with an empty text body, Graph refusing a null one. neverest's own `is_empty_body` workaround stays until it moves to the release carrying this.

Capability moved: events-ical.md, "An event without a description is created with no body".

## Checked

- src/v1/rest/users/events/ical.rs `an_event_without_description_is_created_with_no_body`: failed before the fix (the create body held `"body":{"content":"","contentType":"text"}`), passes after; `an_event_with_description_is_created_with_its_body` passes on both sides.
