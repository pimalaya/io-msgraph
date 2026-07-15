//! Microsoft Graph message attachment resource types.
//!
//! One module per type: the attachment resource and its list page.
//! The modules are an internal organization detail: every type
//! flattens into the attachments path.

mod attachment;
mod attachments_list_response;

#[doc(inline)]
pub use attachment::MsgraphAttachment;
#[doc(inline)]
pub use attachments_list_response::MsgraphAttachmentsListResponse;
