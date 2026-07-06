//! Microsoft Graph REST API, mirroring the reference resource tree. The
//! mail and contacts surfaces hang off the `users` resource (`/me` or
//! `/users/{id}`): mail folders, messages, the sendMail action, contact
//! folders and contacts.
//!
//! <https://learn.microsoft.com/en-us/graph/api/overview>

pub mod users;
