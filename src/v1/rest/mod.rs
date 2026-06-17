//! Microsoft Graph REST API, mirroring the reference resource tree. The
//! mail surface hangs off the `users` resource (`/me` or
//! `/users/{id}`): mail folders, messages and the sendMail action.
//!
//! <https://learn.microsoft.com/en-us/graph/api/overview>

pub mod users;
