---
cairn: change
id: api-errors-and-pages
status: landed
created: 2026-11-13
---

# API errors and pages

## Why

Third batch from the family audit of 2026-11-13. What a Graph answer says beyond its body was lost, and three request shapes let a caller do the wrong thing without knowing.

- **Headers** (MSGRAPH-03). A non-2xx answer kept the status, the code and the message. A throttled request lost its `Retry-After`, which an I/O-free core cannot wait out itself, a 401 lost the claims challenge the next token request has to answer, and no error said whether the connection survived (the audit reports a consumer wrapping its stream in a watcher to find out). A 2xx answer kept no header either.
- **Retries** (MSGRAPH-17). `is_retryable` answered true for a 5xx whatever the method: a caller looping on it sends a mail twice when the first send was accepted before the 504.
- **Recipients** (MSGRAPH-06). The four recipient lists of a message were left out of a body when empty, so an update could not clear them: a removed Bcc stayed on the draft and `message_send` delivered to it.
- **Pages** (MSGRAPH-15). Fifteen response types carried a `next_link`; eight coroutines could follow one. Each consumer rebuilt the loop for the other seven.
- **Links** (MSGRAPH-08). The eight `from_link` constructors built an authorized request for whatever URL they were given. Every driver in the family writes to the one Graph stream, so nothing leaked as shipped, but a pump picking its connection from the URL would hand the token to another host.
- **Attachments** (MSGRAPH-16). An attachment of any size was encoded into one POST and learnt Graph's limit from the rejection.
- **HTML bodies** (MSGRAPH-18). `strip_html` kept the text of `style` and `script`, glued paragraphs, and decoded `&amp;` before `&lt;`, so the text `&lt;` read as `<`.

## What

- `MsgraphSendError::Api` holds a boxed `MsgraphApiError`: status, code, message, method, `retry_after`, `www_authenticate`, `request_id`, `keep_alive`. `MsgraphSendOutput` gains `headers`.
- `is_retryable` is true for a 429, and for a 500, 502, 503 or 504 on a GET, HEAD, PUT or DELETE. `is_throttled` tells the 429 alone.
- The recipient lists are `MsgraphField<Vec<MsgraphRecipient>>`, as the categories became on 2026-11-12.
- `v1::page` holds `MsgraphPage<T>` and the `MsgraphPageGet<T>` coroutine. The fifteen response types are aliases of the page, so the first page and the ones that follow are one type. `MsgraphClientStd::page_get` wraps the coroutine.
- A link is parsed by one function that refuses any origin but that of `MSGRAPH_API_BASE`; `MsgraphPageGet` and the eight `from_link` go through it.
- `MsgraphAttachmentCreate::new` refuses a content over `MSGRAPH_ATTACHMENT_MAX_LEN` (3 MiB).
- `strip_html` skips what a head, a style, a script and a comment hold, breaks the line at a `br` and at the end of a `p` or a `div`, and reads character references in one pass.

## Not done

The upload session for larger attachments, and a bounded page walker in the client layer, which the audit named as optional.
