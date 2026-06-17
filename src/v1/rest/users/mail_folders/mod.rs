//! Microsoft Graph mail folders (`users.mailFolders`): list, get,
//! create, delete.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/mailfolder>

mod types;
#[doc(inline)]
pub use types::*;

pub mod create;
pub mod delete;
pub mod get;
pub mod list;
