---
cairn: log
change: messages-list-count
landed: 2026-11-12
---

# A counted message listing carries its count

`MsgraphMessagesListParams::count` asked Graph for `$count`, but `MsgraphMessagesListResponse` dropped the `@odata.count` it answered, so no caller could read it (Himalaya's `msgraph messages list --count` printed nothing). The response now carries it as `count`, `None` when not asked for.

Test `a_counted_page_carries_its_count`. Additive, landed without a proposal.

Capabilities moved: none written yet (messages have no spec file).
