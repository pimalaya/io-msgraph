//! Microsoft Graph messages (`users.messages`): list, get (JSON and raw
//! MIME), create draft, update, delete, move, copy and send.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/message>

mod types;
#[doc(inline)]
pub use types::*;

pub mod copy;
pub mod create;
pub mod delete;
pub mod get;
pub mod get_raw;
pub mod list;
pub mod move_to;
pub mod send;
pub mod update;
