//! Paged collections: the page every listing and every delta round
//! answers ([`MsgraphPage`]) and the coroutine following the links it
//! carries ([`MsgraphPageGet`]).
//!
//! <https://learn.microsoft.com/en-us/graph/paging>

use alloc::{format, string::String, vec::Vec};

use io_http::rfc6750::bearer::HttpAuthBearer;
use log::{debug, trace};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use url::Url;

use crate::{
    coroutine::*,
    msgraph_try,
    v1::send::{MSGRAPH_API_BASE, MsgraphSend, MsgraphSendError, MsgraphSendOutput},
};

/// One page of a collection: its items and the OData links to what
/// follows.
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct MsgraphPage<T> {
    /// The items of the page.
    #[serde(default = "Vec::new")]
    pub value: Vec<T>,
    /// The URL of the next page, when one exists.
    #[serde(default, rename = "@odata.nextLink")]
    pub next_link: Option<String>,
    /// The URL closing a delta round, carrying the token of the next one.
    #[serde(
        default,
        rename = "@odata.deltaLink",
        skip_serializing_if = "Option::is_none"
    )]
    pub delta_link: Option<String>,
    /// The number of items matching the query, when `$count` asked for
    /// it.
    #[serde(
        default,
        rename = "@odata.count",
        skip_serializing_if = "Option::is_none"
    )]
    pub count: Option<u64>,
}

impl<T> Default for MsgraphPage<T> {
    fn default() -> Self {
        Self {
            value: Vec::new(),
            next_link: None,
            delta_link: None,
            count: None,
        }
    }
}

/// Gets the page of `T` an `@odata.nextLink` or an `@odata.deltaLink`
/// names.
pub struct MsgraphPageGet<T> {
    send: MsgraphSend<MsgraphPage<T>>,
}

impl<T: DeserializeOwned> MsgraphPageGet<T> {
    /// Gets the page behind `link`, sent as it is: it already carries
    /// the query and the token Graph issued.
    ///
    /// A link off the origin of [`MSGRAPH_API_BASE`] is refused with
    /// [`MsgraphSendError::InvalidRequest`].
    pub fn new(auth: &HttpAuthBearer, link: &str) -> Result<Self, MsgraphSendError> {
        debug!("prepare microsoft graph page retrieval");
        trace!("link: {link:?}");

        let send = MsgraphSend::get(auth, link_url(link)?);

        Ok(Self { send })
    }

    /// Asks Graph for at most `size` items per page
    /// (`Prefer: odata.maxpagesize={size}`), see
    /// [`MsgraphSend::max_page_size`].
    pub fn max_page_size(mut self, size: u32) -> Self {
        self.send = self.send.max_page_size(size);
        self
    }
}

impl<T: DeserializeOwned> MsgraphCoroutine for MsgraphPageGet<T> {
    type Yield = MsgraphYield;
    type Return = Result<MsgraphSendOutput<MsgraphPage<T>>, MsgraphSendError>;

    fn resume(&mut self, arg: Option<&[u8]>) -> MsgraphCoroutineState<Self::Yield, Self::Return> {
        let out = msgraph_try!(&mut self.send, arg);
        debug!("page received");
        trace!("items: {}", out.response.value.len());
        MsgraphCoroutineState::Complete(Ok(out))
    }
}

/// Parses a link Graph issued, refused unless it stays on the origin of
/// [`MSGRAPH_API_BASE`]: the request it builds carries the bearer token.
pub(crate) fn link_url(link: &str) -> Result<Url, MsgraphSendError> {
    let url = Url::parse(link)?;
    let origin = url.origin();

    if origin != Url::parse(MSGRAPH_API_BASE)?.origin() {
        let origin = origin.ascii_serialization();
        let reason = format!("a link to `{origin}` leaves the Microsoft Graph origin");
        return Err(MsgraphSendError::InvalidRequest(reason));
    }

    Ok(url)
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::v1::page::MsgraphPage;

    #[test]
    fn a_page_leaves_out_the_links_and_the_count_it_does_not_hold() {
        let page = MsgraphPage::<String> {
            value: [String::from("a")].into(),
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_string(&page).unwrap(),
            r#"{"value":["a"],"@odata.nextLink":null}"#
        );

        let read: MsgraphPage<String> =
            serde_json::from_str(r#"{ "@odata.deltaLink": "d", "@odata.count": 3 }"#).unwrap();
        assert!(read.value.is_empty());
        assert_eq!(read.delta_link.as_deref(), Some("d"));
        assert_eq!(read.count, Some(3));
    }
}
