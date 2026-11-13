---
cairn: tasks
change: api-errors-and-pages
---

- [x] `MsgraphApiError` with the headers a caller acts on, filled by the JSON send, the two raw sends and a batch item; `MsgraphSendOutput::headers`
- [x] `is_retryable` by method, `is_throttled`, red test on a 503 answering a mail send
- [x] Tri-state recipient lists, red test on a cleared list missing from the PATCH
- [x] `MsgraphPage<T>`, the fifteen response aliases, `MsgraphPageGet<T>`, `MsgraphClientStd::page_get`
- [x] One link parser with the origin check, behind `MsgraphPageGet` and the eight `from_link`, red test on four foreign links
- [x] Attachment size guard, red test at the limit and one octet over
- [x] `strip_html` rewritten, red test on a styled document
- [x] CHANGELOG.md, src/lib.rs header, fold into cairn/spec (errors, paging, messages, events-ical), log, archive
- [ ] Upload session for attachments over the limit
- [ ] Bounded page walker in the client layer
