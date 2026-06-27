//! Microsoft Graph mail folders (`users.mailFolders`): list, get,
//! create, update, delete, copy, move, list child folders.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/mailfolder>

mod types;
#[doc(inline)]
pub use types::*;

pub mod child_folders;
pub mod copy;
pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod r#move;
pub mod update;
