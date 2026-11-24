---
cairn: change
id: the-live-night-findings
status: landed
created: 2026-11-24
---

# The live night findings

The live test night of 2026-11-23 (workspace .lanes/live/report.md §9) and the neverest fix round of 2026-11-24 (neverest 8d44f88, its cairn/log/2026-11-24-a-graph-event-created-without-a-body.md) found two gaps in io-msgraph 0.10.0.

## What

1. **A create always sets a body.** `MsgraphEvent::create_from_ical` sends an empty text body for an event without DESCRIPTION. Exchange rewrites an event 0.5 to 4 seconds after a write that sets its body, even an empty one, so a delete queued right after the create meets a server edit and loses (pimdir SYNC §5). neverest strips the empty body itself today; the workaround stays until neverest moves to the release carrying this fix. Recommendation: leave an empty body out of the create body; an update keeps clearing a removed description with an empty text body, Graph refusing a null one.
2. **No online meeting provider.** `MsgraphEvent` lacks `onlineMeetingProvider` and `MsgraphCalendar` lacks `allowedOnlineMeetingProviders` and `defaultOnlineMeetingProvider`, so neverest (task N7) can neither ask for a provider nor tell why a calendar holds no online meeting. Recommendation: add the three fields over one `MsgraphOnlineMeetingProviderType`, a provider this version does not know reading as `unknown`.

## Not in scope

An event created or updated with a description is still rewritten; that is neverest's to report (its ruling R2, a delete overruled by a server edit). The To Do task projection sends an empty body the same way, which needs no change: probed 2026-11-24, To Do moves no etag or `lastModifiedDateTime` over 23 s after a create with an empty, absent or text body, and a create with no body reads back an empty text body.
