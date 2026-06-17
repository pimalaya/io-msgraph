//! Microsoft Graph user resource (`users`, or the `me` shortcut),
//! including its mail folders, messages and the sendMail action.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/user>

mod types;
#[doc(inline)]
pub use types::*;

pub mod get;
pub mod mail_folders;
pub mod messages;
pub mod send_mail;
