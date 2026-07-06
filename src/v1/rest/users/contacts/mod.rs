//! Microsoft Graph contacts (`users.contacts`): list, get, create,
//! update, delete.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/contact>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;
