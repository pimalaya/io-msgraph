# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-15

### Added

- Added the I/O-free coroutine core for the Microsoft Graph API v1.0: the `MsgraphCoroutine` contract, the shared `MsgraphSend` HTTP/JSON primitive with the Graph error envelope parsing, and the OData query serializer.
- Added the mail surface under `v1::rest::users`: mail folders (list, get, create, update, delete, move, copy, list child folders), messages (list, get, get raw MIME, create draft from JSON or MIME, update, delete, move, copy, send), attachments (list, create, get raw content, delete) and the sendMail action (JSON and MIME form).
- Added the contacts surface under `v1::rest::users`: contact folders (list, get, create, update, delete, list child folders) and contacts (list, get, create, update, delete, track changes with delta).
- Added `MsgraphClientStd` (`client` feature): a std blocking client with one convenience method per operation, and a `connect` constructor opening graph.microsoft.com through pimalaya-stream (`rustls-ring` default, `rustls-aws`, `native-tls`).

[unreleased]: https://github.com/pimalaya/io-msgraph/compare/v0.1.0..HEAD
[0.1.0]: https://github.com/pimalaya/io-msgraph/compare/root..v0.1.0
