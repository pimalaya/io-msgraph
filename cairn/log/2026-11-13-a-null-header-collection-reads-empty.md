---
cairn: log
change: a-null-header-collection-reads-empty
landed: 2026-11-13
---

# A null header collection reads empty

`MsgraphMessage::internet_message_headers` took a missing member and refused `null` (`invalid type: null, expected a sequence`), which fails the whole page a message is listed on. What Graph answers for a draft under `$select=internetMessageHeaders` was not seen on the test tenant, whose Drafts folder held none, and neverest now selects the member on every listing of a folder, so one such row would have failed that folder for good. The member reads `null` as an empty list (`null_as_empty`, a private deserializer of src/v1/rest/users/messages.rs), which is what an empty list already meant: no header field returned.

Test `messages_list_selects_internet_message_headers` gains a row answering `null`; the refusal was seen from neverest on 9a203bb. Landed without a proposal, a fix of the day's own change.

Capabilities moved: messages ("A message carries its Internet header fields as returned").
