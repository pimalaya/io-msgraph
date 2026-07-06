//! Microsoft Graph user resource (`users`, or the `me` shortcut),
//! including its mail folders, messages, the sendMail action, contact
//! folders and contacts.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/user>

mod types;
#[doc(inline)]
pub use types::*;

pub mod contact_folders;
pub mod contacts;
pub mod get;
pub mod mail_folders;
pub mod messages;
pub mod send_mail;
