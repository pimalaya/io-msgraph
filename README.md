# I/O Microsoft Graph [![Documentation](https://img.shields.io/docsrs/io-msgraph?style=flat&logo=docs.rs&logoColor=white)](https://docs.rs/io-msgraph/latest/io_msgraph) [![Matrix](https://img.shields.io/badge/chat-%23pimalaya-blue?style=flat&logo=matrix&logoColor=white)](https://matrix.to/#/#pimalaya:matrix.org) [![Mastodon](https://img.shields.io/badge/news-%40pimalaya-blue?style=flat&logo=mastodon&logoColor=white)](https://fosstodon.org/@pimalaya)

Microsoft Graph API client library for Rust.

https://learn.microsoft.com/en-us/graph/api/overview

It currently covers the mail and contacts surfaces; see [API coverage](#api-coverage) for the supported and planned Graph domains.

## Table of contents

- [API coverage](#api-coverage)
- [Usage](#usage)
- [Examples](#examples)
- [License](#license)
- [AI disclosure](#ai-disclosure)
- [Contributing](CONTRIBUTING.md)
- [Social](#social)
- [Sponsoring](#sponsoring)

## API coverage

Microsoft Graph is a single API spanning many domains; io-msgraph covers them incrementally. This table tracks what is implemented today.

| Domain     | Coverage                                                                                                                                                              | Status    |
|------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------|
| [Mail]     | mail folders (list, get, create, update, delete); messages (list, get, get raw MIME, create draft from JSON or MIME, update, delete, move, copy, send); attachments (list, get raw content, delete); sendMail (JSON and MIME); signed-in user (`me`) | Supported |
| [Calendar] | events and calendars                                                                                                                                                 | Soon      |
| [Contacts] | contact folders (list, get, create, update, delete, list child folders); contacts (list, get, create, update, delete)                                               | Supported |

[Mail]: https://learn.microsoft.com/en-us/graph/outlook-mail-concept-overview
[Calendar]: https://learn.microsoft.com/en-us/graph/outlook-calendar-concept-overview
[Contacts]: https://learn.microsoft.com/en-us/graph/outlook-contacts-concept-overview

## Usage

I/O Microsoft Graph can be consumed three ways, depending on how much of the I/O stack you want to own. Each mode is gated by cargo features.

> [!TIP]
> I/O Microsoft Graph is written in [Rust](https://www.rust-lang.org/) and uses [cargo features](https://doc.rust-lang.org/cargo/reference/features.html) to gate the client layers. The default feature set is declared in [Cargo.toml](./Cargo.toml) or on [docs.rs](https://docs.rs/crate/io-msgraph/latest/features).

### Full client

If you want a ready-to-use, standard, blocking client with TCP connection and TLS negociation managed for you:

```toml,ignore
[dependencies]
io-msgraph = "0.0.1" # rustls-ring is enabled by default
```

```rust,no_run
use io_msgraph::v1::client::MsgraphClientStd;

let mut client = MsgraphClientStd::connect("token", Default::default()).unwrap();

let out = client.me().unwrap();
println!("User: {:?}", out.response.user_principal_name);

let out = client.mail_folders_list(&Default::default()).unwrap();
for folder in &out.response.value {
    println!("{}: {}", folder.id, folder.display_name);
}
```

### Light client

If you still want a standard, blocking client but you want to manage TCP and TLS on your own:

```toml,ignore
[dependencies]
io-msgraph = { version = "0.0.1", default-features = false, features = ["client"] }
rustls = "0.23"
rustls-platform-verifier = "0.7"
```

```rust,no_run
use std::{net::TcpStream, sync::Arc};

use io_msgraph::v1::client::MsgraphClientStd;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use rustls_platform_verifier::ConfigVerifierExt;

// TLS config
let config = ClientConfig::with_platform_verifier().unwrap();
let server_name = "graph.microsoft.com".try_into().unwrap();
let conn = ClientConnection::new(Arc::new(config), server_name).unwrap();
let tcp = TcpStream::connect(("graph.microsoft.com", 443)).unwrap();
let stream = StreamOwned::new(conn, tcp);

// Standard, blocking client
let mut client = MsgraphClientStd::new(stream, "token", Default::default());

let out = client.me().unwrap();
println!("User: {:?}", out.response.user_principal_name);
```

### Coroutines

Otherwise you can build your own client using I/O-free coroutines directly:

```toml,ignore
[dependencies]
io-msgraph = { version = "0.0.1", default-features = false }
```

> [!IMPORTANT]
> For such advanced usage, it is preferable to read the [architecture guide](ARCHITECTURE.md).

## Examples

Have a look at real-world projects built on top of this library:

- [io-email](https://github.com/pimalaya/io-email): Email client library
- [Himalaya CLI](https://github.com/pimalaya/himalaya): CLI to manage emails

## AI disclosure

This project is developed with AI assistance. This section documents how, so users and downstream packagers can make informed decisions.

- **Tools**: Claude Code (Anthropic), Opus 4.8, invoked locally with a persistent project-scoped memory and a small set of repo-specific rules.
- **Used for**: Refactors, mechanical multi-file edits, boilerplate (feature gates, error enums, derive macros, trait impls), test scaffolding, doc polish, exploratory design conversations.
- **Not used for**: Engineering, critical code, git manipulation (commit, merge, rebase…), real-world tests.
- **Verification**: Every AI-assisted change is read, compiled, tested, and formatted before commit (`nix develop --command cargo check / cargo test / cargo fmt`). Behavioural correctness is verified against the Microsoft Graph API reference, not assumed from the model output. Tests are never adjusted to fit AI-generated code; the code is adjusted to fit correct behaviour.
- **Limitations**: AI models occasionally produce code that compiles and passes tests but is subtly wrong: off-by-one errors, missed edge cases, plausible but nonexistent APIs, stale spec references. The verification workflow catches most of this; it does not catch all of it. Bug reports are welcome and taken seriously.
- **Last reviewed**: 17/06/2026

## License

This project is licensed under either of:

- [MIT license](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

## Social

- Chat on [Matrix](https://matrix.to/#/#pimalaya:matrix.org)
- News on [Mastodon](https://fosstodon.org/@pimalaya) or [RSS](https://fosstodon.org/@pimalaya.rss)
- Mail at [pimalaya.org@posteo.net](mailto:pimalaya.org@posteo.net)

## Sponsoring

[![nlnet](https://nlnet.nl/logo/banner-160x60.png)](https://nlnet.nl/)

Special thanks to the [NLnet foundation](https://nlnet.nl/) and the [European Commission](https://www.ngi.eu/) that have been financially supporting the project for years:

- 2022 → 2023: [NGI Assure](https://nlnet.nl/project/Himalaya/)
- 2023 → 2024: [NGI Zero Entrust](https://nlnet.nl/project/Pimalaya/)
- 2024 → 2026: [NGI Zero Core](https://nlnet.nl/project/Pimalaya-PIM/)
- *2027 in preparation…*

If you appreciate the project, feel free to donate using one of the following providers:

[![GitHub](https://img.shields.io/badge/-GitHub%20Sponsors-fafbfc?logo=GitHub%20Sponsors)](https://github.com/sponsors/soywod)
[![Ko-fi](https://img.shields.io/badge/-Ko--fi-ff5e5a?logo=Ko-fi&logoColor=ffffff)](https://ko-fi.com/soywod)
[![Buy Me a Coffee](https://img.shields.io/badge/-Buy%20Me%20a%20Coffee-ffdd00?logo=Buy%20Me%20A%20Coffee&logoColor=000000)](https://www.buymeacoffee.com/soywod)
[![Liberapay](https://img.shields.io/badge/-Liberapay-f6c915?logo=Liberapay&logoColor=222222)](https://liberapay.com/soywod)
[![PayPal](https://img.shields.io/badge/-PayPal-0079c1?logo=PayPal&logoColor=ffffff)](https://www.paypal.com/paypalme/soywod)
