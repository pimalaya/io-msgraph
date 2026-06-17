//! Microsoft Graph messages (`users.messages`): list, get (JSON and raw
//! MIME), create draft (JSON and raw MIME), update, delete, move, copy,
//! send and the nested attachments collection.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/message>

mod types;
#[doc(inline)]
pub use types::*;

pub mod attachments;
pub mod copy;
pub mod create;
pub mod create_mime;
pub mod delete;
pub mod get;
pub mod get_raw;
pub mod list;
pub mod move_to;
pub mod send;
pub mod update;
