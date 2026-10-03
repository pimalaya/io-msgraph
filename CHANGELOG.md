# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added `MsgraphPatternedRecurrence::bounds` (`ical` feature): the dates a series spans, following its range type. Graph fills the `endDate` of a `numbered` or `noEnd` range with `0001-01-01`, so reading `endDate` alone gave an instances window ending before it starts.

### Changed

- Documented that Graph returns `cancelledOccurrences` only on a read of a series master by id, never in a listing.

## [0.4.4] - 2026-10-02

### Fixed

- Fixed `update_from_ical` patching an event's body to `null`, which Graph refuses (HTTP 400, "The body of the item is invalid"): Graph reads back an empty HTML body, so any edit of an event without DESCRIPTION, or one clearing it, failed. A missing DESCRIPTION is now an empty text body, and bodies compare by their text.

## [0.4.3] - 2026-10-02

### Fixed

- Fixed `to_ical` writing an event in UTC: Graph answers a read in UTC, so a series created in Europe/Paris read back anchored on UTC and drifted by an hour at each DST change once expanded. `MSGRAPH_EVENT_ICAL_SELECT` now selects `originalStartTimeZone` and `originalEndTimeZone`, and the projection tells a UTC start and end in that zone when the database knows it.

- Fixed `from_ical` dropping an edited DESCRIPTION: a document read from Graph carries Exchange's HTML body as `X-ALT-DESC` beside the text, and the HTML always won, so `update_from_ical` saw no change and left the edit out of the patch. The HTML now wins only while it says what the text says.

- Documented that `to_ical_series` needs its exceptions read with `MSGRAPH_EVENT_ICAL_SELECT`: Graph's default instance listing leaves out `originalStart`, and an exception without it gets no RECURRENCE-ID.

## [0.4.2] - 2026-10-01

### Added

- Added the `ical` feature: `MsgraphEvent::to_ical`, `to_ical_series`, `from_ical`, `create_from_ical` and `update_from_ical` project an event, or a series with its exceptions, onto an iCalendar document and back.

  The recurrence maps onto an RRULE both ways, a rule Graph cannot hold being refused; cancelled occurrences become EXDATEs and exceptions VEVENTs with a RECURRENCE-ID. Windows zone names read as their CLDR IANA counterpart and every zone gets a VTIMEZONE, through ical-rs's `tzdb` feature. The UID and every unmanaged line ride a stash extended property, read back with `MSGRAPH_EVENT_STASH_EXPAND`. Only the series master is written back.

- Added `MsgraphEvent::cancelled_occurrences`.

- Added calendars and events: `calendars` (list, get, create, update, delete) and `events` (list, get, create, update, delete, the instances of a series, the calendar view and its delta), with `MsgraphCalendar`, `MsgraphEvent` and the recurrence, attendee and time types, and a client method each.

  An event's times keep the zone Graph returns, a Windows or an IANA name; the calendar view and its delta expand series within a window, the events listing returns series masters.

- Added `MsgraphContact::stashed_uid`, the vCard UID a contact's stash carries, for a sync engine to check that a write kept it.

### Changed

- The vCard UID rides the stash, so a contact keeps the UID it was written with. **Behaviour change.**

  It used to be dropped on write and minted from the Graph id on read, which gave one person two identities across a sync. A contact with no stashed UID, created by Graph itself or by an earlier version, still reads back with one minted from its Graph id.

## [0.4.1] - 2026-10-01

### Added

- Added the `vcard` feature: `MsgraphContact::to_vcard`, `from_vcard`, `create_from_vcard` and `update_from_vcard` project a Graph contact onto a vCard 4.0 document and back.

  Graph-only fields ride as read-only `X-MSGRAPH-*` properties, every other line round-trips through a stash extended property read back with `MSGRAPH_CONTACT_STASH_EXPAND`. The projection moved here from Cardamum, stash id included, so contacts Cardamum already stashed still read back.

## [0.4.0] - 2026-09-29

### Added

- Added `MsgraphClientStdConnectOptions::proxy`, tunnelling the connection through a SOCKS5 or HTTP proxy.

## [0.3.1] - 2026-09-29

### Added

- Added JSON batching (`POST /$batch`) and the matching `batch` client method.

### Removed

- Removed the thiserror dependency. Error messages, sources and `From` conversions are unchanged.

## [0.3.0] - 2026-08-15

### Added

- Added `MsgraphContactsDelta::from_link` and the matching `contacts_delta_from_link` client method, resuming a contacts delta round from a saved link.

### Changed

- Bumped pimalaya-stream to 0.3. **Behaviour change.**

  The transport now retries a stream reporting it is not ready for up to a minute, then fails with `TimedOut`, and a read deadline stops a silent server from blocking forever. Consumers must move to the same `Tls` type.

- Bumped io-http to 0.5.
- Raised the MSRV from 1.87 to 1.88.

## [0.2.2] - 2026-08-06

### Added

- Added the messages delta operation (`v1::rest::users::messages::delta`) and the matching `messages_delta` and `messages_delta_from_link` client methods.

## [0.2.1] - 2026-07-25

### Added

- Added the `schemars` feature, deriving `JsonSchema` on the mail output types. It is off by default and stays `no_std`.

## [0.2.0] - 2026-07-16

### Changed

- Moved each resource type into its resource module, dropping the internal `types` submodules.

  Entity types keep their path. `MsgraphField` moved to `v1::field`, and operation companions (`Msgraph*ListResponse`, the contacts delta types) moved into their operation module.

## [0.1.0] - 2026-07-15

### Added

- Added the I/O-free coroutine core for Microsoft Graph v1.0: the `MsgraphCoroutine` contract, the `MsgraphSend` HTTP/JSON primitive and the OData query serializer.
- Added the mail surface: mail folders, messages, attachments and the sendMail action.
- Added the contacts surface: contact folders and contacts, with delta.
- Added `MsgraphClientStd` (`client` feature), a std blocking client with a `connect` constructor behind the TLS features.

[unreleased]: https://github.com/pimalaya/io-msgraph/compare/v0.4.4..HEAD
[0.4.4]: https://github.com/pimalaya/io-msgraph/compare/v0.4.3..v0.4.4
[0.4.3]: https://github.com/pimalaya/io-msgraph/compare/v0.4.2..v0.4.3
[0.4.2]: https://github.com/pimalaya/io-msgraph/compare/v0.4.1..v0.4.2
[0.4.1]: https://github.com/pimalaya/io-msgraph/compare/v0.4.0..v0.4.1
[0.4.0]: https://github.com/pimalaya/io-msgraph/compare/v0.3.1..v0.4.0
[0.3.1]: https://github.com/pimalaya/io-msgraph/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/pimalaya/io-msgraph/compare/v0.2.2..v0.3.0
[0.2.2]: https://github.com/pimalaya/io-msgraph/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/pimalaya/io-msgraph/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/io-msgraph/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/io-msgraph/compare/root..v0.1.0
