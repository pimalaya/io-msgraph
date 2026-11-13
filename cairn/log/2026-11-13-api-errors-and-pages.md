---
cairn: log
change: api-errors-and-pages
landed: 2026-11-13
---

# API errors and pages

Third batch from the family audit of 2026-11-13: MSGRAPH-03, MSGRAPH-17, MSGRAPH-06, MSGRAPH-15 with the origin check of MSGRAPH-08, MSGRAPH-16 and MSGRAPH-18.

**API error** (MSGRAPH-03). `MsgraphSendError::Api` was a struct variant of three fields; it now holds a `MsgraphApiError`, boxed because the eight fields put every `Result` of the client over clippy's `result_large_err`. `MsgraphApiError::parse` reads the envelope (the `request-id` of its `innerError` too) and the headers, any iterator of name and value pairs so the JSON send, the two raw `$value` sends and a batch item share it; the exchange adds the method and the keep-alive flag. `Retry-After` is kept in its seconds form as a `Duration`, a date reading as none, as `MsgraphBatchResponse::retry_after` already did. A batch item has no connection and no method of its own, so its error reads as reusable and holds an empty method. `MsgraphSendOutput` gained `headers`, moved out of the response, not cloned. Tests `an_api_error_carries_what_its_headers_ask` and `a_response_carries_its_headers`; neither could be written against the old type, which had no such field.

**Retries** (MSGRAPH-17). `is_retryable` is a 429, or a 500, 502, 503 or 504 on a GET, HEAD, PUT or DELETE. PATCH is left out with POST: HTTP does not make it idempotent, and a caller that knows its PATCH is can still read the status. Test `a_server_error_is_retryable_only_where_a_repeat_is_safe`, which showed a 503 answering a mail send as retryable.

**Recipients** (MSGRAPH-06). The four lists follow the categories of 2026-11-12. Test `a_cleared_recipient_list_reaches_the_patch`, first written with empty `Vec`s against the old type, where the PATCH body held none of the four names.

**Pages** (MSGRAPH-15). `MsgraphPage<T>` replaces fifteen structs of the same shape; their names stay as type aliases, so a consumer naming `MsgraphMailFoldersListResponse` still compiles and the page `MsgraphPageGet<MsgraphMailFolder>` answers is that same type. The one page holds the delta link and the count for every collection; both are left out of a serialised page that has none, which changes what a delta page without a delta link serialises to (no `"@odata.deltaLink":null`). `Default` is written by hand and `value` defaults through `Vec::new`, so neither asks `T: Default`. Tests `a_listing_follows_its_next_link` and `a_page_leaves_out_the_links_and_the_count_it_does_not_hold`.

**Links** (MSGRAPH-08). `link_url` of src/v1/page.rs compares `Url::origin`, so the scheme, the host and the port all count, and its refusal names the origin, not the link, which holds the delta token. Test `rejects_a_link_off_the_graph_origin`: four foreign links through the eight `from_link`, all accepted before.

**Attachments** (MSGRAPH-16). The limit is 3 MiB of content, the figure of Graph's page on large attachments as remembered: neither the page nor the service was consulted for this change. Test `rejects_an_attachment_over_what_one_request_takes`.

**HTML bodies** (MSGRAPH-18). `strip_html` walks tags instead of characters. After a tag that breaks the line, the line break the source put right behind it is skipped, so `<br>\r\n` is one break; other source line breaks stay as they were, the HTML Exchange emits not being at hand to justify collapsing them. A reference is looked for in the nine octets after its `&`, which keeps a body full of bare ampersands linear. Test `an_html_body_reads_as_the_text_it_shows`, which showed `tp {margin:0}var a = 1  0;Hithere` for `Hi\nthere`.

Existing tests changed: the three that matched `MsgraphSendError::Api { .. }` read the new shape, and the live suite builds its recipients as `MsgraphField::Set`.

Left open: the upload session, a bounded page walker in the client layer, and whether the fifteen aliases should stay once the consumers name `MsgraphPage` themselves.

Capabilities moved: errors and paging (new files, by accretion), messages (two requirements added), events-ical (one added).
