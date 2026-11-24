---
cairn: spec
capability: events
status: current
---

# Events and calendars

The calendar and event surface (`users/{id}/calendars` and `users/{id}/events`, Graph's `calendar` and `event`), as typed coroutines over `MsgraphCalendar` and `MsgraphEvent`. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/rest/users/calendars.rs and src/v1/rest/users/events.rs headers holding the rest. The iCalendar projection has its own file, events-ical.md.

### Requirement: An event names its online meeting provider
An event SHALL carry `onlineMeetingProvider` as `MsgraphOnlineMeetingProviderType` (`teamsForBusiness`, `skypeForBusiness`, `skypeForConsumer`, `unknown`), tri-state like every writable field: left out of a written body when Unset, so a create or an update sends it only when its writer sets it. A calendar SHALL carry, read-only, `allowedOnlineMeetingProviders`, an empty list read as none allowed and kept apart from a member Graph did not return, and `defaultOnlineMeetingProvider`. A provider value this version does not know SHALL read as `unknown`, so one new provider never fails the event or the calendar page it is listed on.

#### Scenario: A tenant that allows no provider
- GIVEN a calendar Graph answers with `"allowedOnlineMeetingProviders": []` and `"defaultOnlineMeetingProvider": "unknown"`, as the Pimalaya test tenant's do (read 2026-11-24)
- WHEN it is read
- THEN its allowed providers are an empty list and its default is `Unknown`
