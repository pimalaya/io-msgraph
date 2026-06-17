# Architecture guide

Read the [Pimalaya ARCHITECTURE](https://github.com/pimalaya/.github/blob/master/ARCHITECTURE.md) first: it describes the conventions every Pimalaya repository shares (the sans-I/O coroutine approach, `no_std`, module and error rules, code style, licensing). This document only covers what is specific to io-msgraph, and assumes you know that shared context.

If a statement here conflicts with the code, the code wins; please flag it.

## Where io-msgraph fits

io-msgraph is a **protocol library**: a set of I/O-free coroutines for the [Microsoft Graph API](https://learn.microsoft.com/en-us/graph/api/overview). It sits one layer above [io-http](https://github.com/pimalaya/io-http) (HTTP/1.1) and [pimalaya-stream](https://github.com/pimalaya/stream) (TCP + TLS), and is consumed by [io-email](https://github.com/pimalaya/io-email) as the Microsoft Graph backend of the shared email domain API, which [himalaya](https://github.com/pimalaya/himalaya) drives through its cross-protocol commands (a dedicated `msgraph` command may be added later). It is the Microsoft Graph analogue of [io-gmail](https://github.com/pimalaya/io-gmail): same shape (JSON over HTTP), different vendor.

Unlike io-gmail, which is scoped to Gmail's mail API, io-msgraph represents the **whole Microsoft Graph API**. This first release covers only the mail surface; sibling resources (calendars, contacts, drives, ...) will be added under the same tree over time.

The crate has two of the three standard layers; there is no CLI:

1. **I/O-free coroutines** (`no_std` core, always present): the Microsoft Graph REST logic.
2. **Std client** (`client` feature): a blocking driver, `MsgraphClientStd`, with `connect` gated behind a TLS feature (`rustls-ring` default, `rustls-aws`, `native-tls`).

## API versioning: everything lives under `v1`

The Microsoft Graph API is versioned (`/v1.0/`), so the crate is too. The version-agnostic primitives stay at the crate root; everything that is v1.0-specific lives under `src/v1/`. The day a breaking version ships, a sibling module is added without breaking `v1` consumers.

- `src/lib.rs`, `src/coroutine.rs`: crate root, shared across versions.
- `src/v1/`: the v1.0 surface (`send.rs`, `query.rs`, `client.rs`, and the whole `rest/` tree).

Callers always import through the version, e.g. `io_msgraph::v1::rest::users::messages::MsgraphMessage`, `io_msgraph::v1::client::MsgraphClientStd`.

## The send primitive

Like io-gmail, every Microsoft Graph call is an independent HTTP request/response, so io-msgraph has a single shared primitive that the JSON coroutines delegate to: `v1::send::MsgraphSend<T>` (`src/v1/send.rs`). It wraps io-http's `Http11Send`, builds the request (the `Authorization` header from the caller's bearer token via `HttpAuthBearer::to_authorization`, `Accept: application/json`, an optional body), and on completion either deserialises the 2xx body into `T` or parses Graph's JSON error envelope (`{ "error": { "code", "message" } }`, where `code` is a string) into `MsgraphSendError::Api { status, code, message }`. A 3xx surfaces as `MsgraphSendError::UnexpectedRedirect`. `MsgraphSend<T>` exposes `get` / `post_json` / `patch_json` / `post_text` / `delete` / `with_method` constructors.

Its terminal value is `MsgraphSendOutput<T> { response: T, keep_alive: bool }`. `keep_alive` lets a driver reuse the TCP/TLS connection across the many small requests a Graph session makes. Empty 2xx bodies (DELETE, sendMail, send draft) deserialise into the `MsgraphNoResponse` unit marker.

The one exception is `messages::get_raw` (`GET /me/messages/{id}/$value`), which returns raw RFC 5322 MIME rather than JSON. It does not use `MsgraphSend<T>`; it drives `Http11Send` directly and yields the response body bytes, reusing `parse_api_error` for the failure case.

## The coroutine contract

io-msgraph follows the standard Pimalaya coroutine shape with crate-local names (`src/coroutine.rs`, version-agnostic):

- trait `MsgraphCoroutine` with `resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return>`;
- `MsgraphCoroutineState` is `Yielded(Y)` or `Complete(R)`;
- the standard yield is `MsgraphYield { WantsRead, WantsWrite(Vec<u8>) }` (a Graph call is I/O-only: no clock, randomness or filesystem);
- the `msgraph_try!` macro is the coroutine `?`: it forwards `Yielded` and short-circuits `Complete(Err(_))`.

Every coroutine is a thin, single-step wrapper holding the send directly (`struct MsgraphX { send: MsgraphSend<T> }`); `new(auth, user_id, ...)` builds the URL and body, and `resume` just delegates. The canonical reference template is the `mail_folders` module. A multi-variant `State` enum is reserved for genuine multi-step coroutines (none yet in the mail subset).

### Logging

Each coroutine logs at two levels, via `use log::{debug, trace};`: `debug!` carries the short human-readable phrase, and a `trace!` directly below dumps the data when there is any. `new()` opens with `debug!("prepare microsoft graph <thing> <op>")` followed by one `trace!` per input variable; `resume()`, once the send resolves, does `debug!("microsoft graph <thing> <verbed>")` then `trace!("out: {out:?}")`. Messages start lowercase and carry no trailing period.

## Authentication

io-msgraph does no OAuth itself. The Graph API only accepts OAuth 2.0 Bearer tokens, so the credential is exactly a bare access token: the std client takes it as `impl ToString` and stores an `io_http::rfc6750::bearer::HttpAuthBearer`; coroutines take `auth: &HttpAuthBearer`; and `send` adds the `Bearer ` prefix via `auth.to_authorization()`. Tokens are short-lived; minting and refreshing them is the caller's responsibility. The base URL is fixed (`MSGRAPH_API_BASE`, `https://graph.microsoft.com/v1.0/`); the mailbox owner is addressed by `v1::send::user_path`, which yields `me` for the authenticated user or `users/{id}` for an explicit user id or principal name (Graph rejects `users/me`).

## Module layout: `v1/rest` mirrors the Graph tree

`src/v1/rest/` mirrors the Graph reference. The mail surface hangs off the `users` resource, so it lives under `rest/users/`, with each sub-resource a directory and each method a file named after the operation:

```
src/
  lib.rs            crate root: no_std, `pub mod coroutine; pub mod v1;`
  coroutine.rs      MsgraphCoroutine / MsgraphCoroutineState / MsgraphYield + msgraph_try!
  v1/
    mod.rs
    send.rs         MsgraphSend<T>, error/output, MsgraphNoResponse, base URL, user_path
    query.rs        Serialize-struct -> OData query pairs (no_std serde serializer)
    client.rs       (client) MsgraphClientStd: boxed stream + auth + user_id
    rest/
      users/        the user (`me`) and its mail surface
        get.rs      GET /me (profile smoke test)
        mail_folders/  list, get, create, delete (+ types)
        messages/   list, get, get_raw ($value MIME), create, update, delete,
                    move_to, copy, send (+ types)
        send_mail.rs   POST /me/sendMail in JSON and MIME form
```

Each directory follows the standard module rules: a private `types` submodule re-exported via `#[doc(inline)] pub use types::*;` in `mod.rs`, then one file per method. `mod.rs` holds only module declarations.

## Types: a faithful mapping of the resource

Domain types are `Msgraph`-prefixed (`MsgraphMessage`, `MsgraphMailFolder`, `MsgraphRecipient`, ...), are never re-exported at the crate root (callers use module-qualified paths), and mirror the Graph schema. Full-resource bodies (`MsgraphMailFolder`, `MsgraphMessage`) are `Default` with `skip_serializing_if` on every optional field, so the same type serves as the create/update body. Wire enums are `Msgraph`-prefixed (`MsgraphImportance`, `MsgraphFlagStatus`, `MsgraphBodyType`). List endpoints take a borrowed `*Params` struct whose fields rename to the OData system query options (`$top`, `$select`, `$filter`, ...), flattened to query pairs by `v1::query::to_query_pairs`.

## The std client

`MsgraphClientStd` (`client` feature, `src/v1/client.rs`) wraps a boxed `Read + Write + Send` stream plus the `HttpAuthBearer` and `user_id`. Its generic `run<C, T>(coroutine)` is the blocking driver loop, returning `MsgraphSendOutput<T>`. It offers one convenience method per operation (`me`, `mail_folders_list`, `message_get`, `message_get_raw`, `send_mail_mime`, ...). `connect(token, MsgraphClientStdConnectOptions)` (TLS features) opens `graph.microsoft.com:443` through pimalaya-stream. The bare bearer token is the only required argument; the rest live in `MsgraphClientStdConnectOptions { tls, user_id }`, which is `Default` (default TLS backend, `user_id` = `me`).

## Testing

`tests/coroutines.rs` runs each coroutine against in-memory HTTP responses (no network): per-operation request/response checks (user, mail folder list/create/delete, message list/update/move, raw `$value`, MIME sendMail), empty-name rejection, OData query serialisation and the Graph error envelope with its fallbacks. `tests/msgraph.rs` is an `#[ignore]`d end-to-end test exercising the full lifecycle (folder and draft create/get/update/move/delete plus sendMail) against the live API, gated behind a TLS feature and driven by `MSGRAPH_ACCESS_TOKEN` (and optional `MSGRAPH_USER_ID`).
