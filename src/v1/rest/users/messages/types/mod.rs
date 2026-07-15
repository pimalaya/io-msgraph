//! Microsoft Graph message resource types.
//!
//! One module per type: the message resource, its body, recipients and
//! flag companions, and its list page. The modules are an internal
//! organization detail: every type flattens into the messages path.

mod body_type;
mod email_address;
mod flag_status;
mod followup_flag;
mod importance;
mod item_body;
mod message;
mod messages_list_response;
mod recipient;

#[doc(inline)]
pub use body_type::MsgraphBodyType;
#[doc(inline)]
pub use email_address::MsgraphEmailAddress;
#[doc(inline)]
pub use flag_status::MsgraphFlagStatus;
#[doc(inline)]
pub use followup_flag::MsgraphFollowupFlag;
#[doc(inline)]
pub use importance::MsgraphImportance;
#[doc(inline)]
pub use item_body::MsgraphItemBody;
#[doc(inline)]
pub use message::MsgraphMessage;
#[doc(inline)]
pub use messages_list_response::MsgraphMessagesListResponse;
#[doc(inline)]
pub use recipient::MsgraphRecipient;
