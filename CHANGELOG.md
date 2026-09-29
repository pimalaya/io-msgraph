# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/pimalaya/io-msgraph/compare/v0.3.0..HEAD
[0.3.0]: https://github.com/pimalaya/io-msgraph/compare/v0.2.2..v0.3.0
[0.2.2]: https://github.com/pimalaya/io-msgraph/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/pimalaya/io-msgraph/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/pimalaya/io-msgraph/compare/v0.1.0..v0.2.0
[0.1.0]: https://github.com/pimalaya/io-msgraph/compare/root..v0.1.0
