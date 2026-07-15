//! Microsoft Graph contact folder resource types.
//!
//! One module per type: the contact folder resource and its list page.
//! The modules are an internal organization detail: every type
//! flattens into the contact_folders path.

mod contact_folder;
mod contact_folders_list_response;

#[doc(inline)]
pub use contact_folder::MsgraphContactFolder;
#[doc(inline)]
pub use contact_folders_list_response::MsgraphContactFoldersListResponse;
