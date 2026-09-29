//! Microsoft Graph REST API, mirroring the reference resource tree. The
//! mail and contacts surfaces hang off the `users` resource (`/me` or
//! `/users/{id}`): mail folders, messages, the sendMail action, contact
//! folders and contacts. JSON batching (`$batch`) sits beside it, at the
//! API root.
//!
//! <https://learn.microsoft.com/en-us/graph/api/overview>

pub mod batch;
pub mod users;
