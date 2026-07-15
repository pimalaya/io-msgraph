//! Microsoft Graph contact resource types.
//!
//! One module per type: the contact resource, its address and extended
//! property companions, and the list and delta pages. The modules are
//! an internal organization detail: every type flattens into the
//! contacts path.

mod contact;
mod contact_delta;
mod contacts_delta_response;
mod contacts_list_response;
mod physical_address;
mod removed;
mod single_value_extended_property;

#[doc(inline)]
pub use contact::MsgraphContact;
#[doc(inline)]
pub use contact_delta::MsgraphContactDelta;
#[doc(inline)]
pub use contacts_delta_response::MsgraphContactsDeltaResponse;
#[doc(inline)]
pub use contacts_list_response::MsgraphContactsListResponse;
#[doc(inline)]
pub use physical_address::MsgraphPhysicalAddress;
#[doc(inline)]
pub use removed::MsgraphRemoved;
#[doc(inline)]
pub use single_value_extended_property::MsgraphSingleValueExtendedProperty;
