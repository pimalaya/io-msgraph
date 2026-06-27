//! Microsoft Graph message attachments
//! (`users.messages.attachments`): create, list, get raw content,
//! delete.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/attachment>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get_raw;
pub mod list;
