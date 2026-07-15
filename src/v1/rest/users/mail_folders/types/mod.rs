//! Microsoft Graph mail folder resource types.
//!
//! One module per type: the mail folder resource and its list page.
//! The modules are an internal organization detail: every type
//! flattens into the mail_folders path.

mod mail_folder;
mod mail_folders_list_response;

#[doc(inline)]
pub use mail_folder::MsgraphMailFolder;
#[doc(inline)]
pub use mail_folders_list_response::MsgraphMailFoldersListResponse;
