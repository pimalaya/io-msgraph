//! Microsoft Graph contacts delta page.
//!
//! One page of a contacts change-tracking round, carrying the changed
//! rows and the paging or closing link.

use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::v1::rest::users::contacts::types::contact_delta::MsgraphContactDelta;

/// One page of a contacts delta round.
///
/// More pages follow through `next_link`; the round ends when
/// `delta_link` arrives (the token of the next round).
#[derive(Debug, Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphContactsDeltaResponse {
    /// The changed contacts of the page.
    #[serde(default)]
    pub value: Vec<MsgraphContactDelta>,
    /// The URL of the next page of the round, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    /// The URL closing the round, carrying the next round's token.
    #[serde(default, rename = "@odata.deltaLink")]
    pub delta_link: Option<String>,
}
