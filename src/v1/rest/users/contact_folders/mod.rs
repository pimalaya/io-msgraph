//! Microsoft Graph contact folders (`users.contactFolders`): list,
//! get, create, update, delete, list child folders.
//!
//! <https://learn.microsoft.com/en-us/graph/api/resources/contactfolder>

mod types;
#[doc(inline)]
pub use types::*;

pub mod child_folders;
pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;
