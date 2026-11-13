---
cairn: log
change: message-internet-headers
landed: 2026-11-13
---

# A message carries its Internet header fields

`MsgraphMessage` modelled no `internetMessageHeaders`, so a caller could name the member in a `$select` and the answer was dropped at deserialization: neverest and the Android app named a Graph message from the decoded `subject`, `sentDateTime` and `from`, where the pimdir standard (STORAGE Annex A.1) reads the raw `Message-ID`, `Subject`, `Date` and `From`. The message now carries `internet_message_headers`, a list of `MsgraphInternetMessageHeader` (`name`, `value`) in the order Graph returns, repeated names kept and values untouched. It is empty when Graph returns none and left out of a written body when empty, as the other collections of the type are. No request selects it on its own: the caller names it in the `$select` it already passes through `MsgraphMessagesListParams`, `MsgraphMessagesDelta::new` or `MsgraphMessagesDeltaParams`.

`MsgraphMessageGet::new` took no query option, so the one read Microsoft documents for the member could not ask for it (neverest builds that URL by hand over `MsgraphSend::get`). It now takes `select` and `expand` as `MsgraphEventGet::new` does, and `MsgraphClientStd::message_get` with it. Breaking for the callers of both, and the new field for a `MsgraphMessage` literal without `..Default::default()`.

What Microsoft's reference says, read on 2026-11-13:

- "message resource type": `internetMessageHeaders` is an `internetMessageHeader` collection, "Requires `$select` to retrieve. Read-only." The same page has custom headers added at create only, named from `x-`, and "Create message" shows such a body, which is why a set list is still serialized.
- "internetMessageHeader resource type": `name` and `value`, two strings.
- "Get message", example 2: `GET /me/messages/{id}?$select=internetMessageHeaders`, the only read of the member the reference shows.
- "List messages" and "message: delta" take a `$select` and say nothing of this member. Whether a listing or a delta page returns it is not verified: no live suite was run for this change.
- Nothing in the reference says which messages carry it. That a sent copy carries none is reported on Microsoft Q&A alone ("How to fetch InternetMessageHeaders for messages in SENT box", 2024-02-06), and whether Graph then leaves the member out or answers an empty list is not known: both read as an empty list.

Probed on the test tenant on 2026-11-13, read-only, five rows each under `$select=id,internetMessageHeaders`: a listing and a delta page of the inbox and of Sent Items all return the member on every row, none `null`. A sent copy carries it there, `From`, `To`, `Subject` and `Date` among its names, so the Q&A answer above does not hold on that tenant. Drafts held no message, so a draft's answer is still unknown.

Tests `messages_list_selects_internet_message_headers` and `message_get_selects_and_expands_only_when_asked`. Landed without a proposal at the maintainer's request.

Capabilities moved: messages (new).
