# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-08-15

### Added

- Added `MsgraphContactsDelta::from_link` and the matching `contacts_delta_from_link` client method, continuing a contacts delta round from an `@odata.nextLink` or starting the next round from a saved `@odata.deltaLink`. The messages delta gained both when it was transposed from the contacts one in 0.2.2; the original never had them, so a contacts round could only ever be opened, never resumed.

### Changed

- Bumped pimalaya-stream to 0.3, which drops the `sasl` module it no longer owns and whose `Read` and `Write` retry a stream reporting it is not ready. **Behaviour change.**

  The `Tls` type this crate takes comes from that version, so a consumer must move with it. A blocking socket is not supposed to report `EAGAIN`, yet callers saw one surface mid-exchange and end the exchange with a bare `Resource temporarily unavailable (os error 35)`, macOS especially. The transport now retries such a failure for a minute before giving up with a `TimedOut` naming the budget, and arms a socket read deadline at connect time so a server going silent on a healthy connection stops blocking the caller forever.

- Bumped io-http to 0.5.
- Raised the minimum supported Rust version from 1.87 to 1.88, following pimalaya-stream and io-http.

## [0.2.2] - 2026-08-06

### Added

- Added the messages delta operation under `v1::rest::users::messages::delta`, transposing the contacts one: an initial coroutine over the whole mailbox or a mail folder (with `$select` support), a `from_link` constructor continuing a round from an `@odata.nextLink` or starting the next round from a saved `@odata.deltaLink`, the delta page response types with the `@removed` marker, and the matching `messages_delta` and `messages_delta_from_link` client methods.

## [0.2.1] - 2026-07-25

### Added

- Added an optional `schemars` feature deriving `schemars::JsonSchema` on the mail output types, so downstream tools can generate JSON Schemas describing Microsoft Graph command output.

  The feature is off by default and stays `no_std`: it pulls only schemars' `derive` (not `std`). It covers the message object (with its recipients, email addresses, body and follow-up flag) and the mail folder and attachment objects.

## [0.2.0] - 2026-07-16

### Changed

- Reorganised the resource types so each lives in its resource module directly, dropping the internal `types` submodules and per-type files together with their flattened re-exports.

  Entity and value-object types keep their existing path (`v1::rest::users::messages::MsgraphMessage`, `MsgraphEmailAddress`, `contacts::MsgraphContact`, `mail_folders::MsgraphMailFolder` and so on). The shared tri-state moved from `v1::MsgraphField` to `v1::field::MsgraphField`. Operation-specific companions moved into their operation module: every `Msgraph*ListResponse` now lives under its resource's `list` module (for example `messages::list::MsgraphMessagesListResponse`, `mail_folders::list::MsgraphMailFoldersListResponse`), and the contacts delta types `MsgraphContactsDeltaResponse`, `MsgraphContactDelta` and `MsgraphRemoved` now live under `contacts::delta`.

## [0.1.0] - 2026-07-15

### Added

- Added the I/O-free coroutine core for the Microsoft Graph API v1.0: the `MsgraphCoroutine` contract, the shared `MsgraphSend` HTTP/JSON primitive with the Graph error envelope parsing, and the OData query serializer.
- Added the mail surface under `v1::rest::users`: mail folders (list, get, create, update, delete, move, copy, list child folders), messages (list, get, get raw MIME, create draft from JSON or MIME, update, delete, move, copy, send), attachments (list, create, get raw content, delete) and the sendMail action (JSON and MIME form).
- Added the contacts surface under `v1::rest::users`: contact folders (list, get, create, update, delete, list child folders) and contacts (list, get, create, update, delete, track changes with delta).
- Added `MsgraphClientStd` (`client` feature): a std blocking client with one convenience method per operation, and a `connect` constructor opening graph.microsoft.com through pimalaya-stream (`rustls-ring` default, `rustls-aws`, `native-tls`).

[unreleased]: https://github.com/pimalaya/io-msgraph/compare/v0.3.0..HEAD
[0.3.0]: https://github.com/pimalaya/io-msgraph/compare/v0.2.2..v0.3.0
[0.2.2]: https://github.com/pimalaya/io-msgraph/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/pimalaya/io-msgraph/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/io-msgraph/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/io-msgraph/compare/root..v0.1.0
