---
cairn: log
change: path-segments-encoded
landed: 2026-11-12
---

# Ids are percent-encoded as one path segment

Every coroutine spliced its ids (folder, message, attachment, event, calendar, contact, To Do list and task ids) raw into the request path, so a value holding `/`, `?` or `#` named another resource: Himalaya's live tests sent `Himalaya-Test-MS-A/Sub` as `/me/mailFolders/Himalaya-Test-MS-A/Sub/messages` and got `Resource not found for the segment 'Sub'`. Each id now goes through the new public `path_segment`, which percent-encodes every byte outside the RFC 3986 `pchar` set and refuses an empty, `.` or `..` segment with `MsgraphSendError::InvalidRequest` (URL resolution drops or climbs those, encoded or not). `user_path` encodes a user id the same way. Graph ids (base64url) pass unchanged.

Tests `a_path_segment_keeps_a_graph_id_and_encodes_separators` and `a_path_segment_refuses_what_url_resolution_drops`. Trivial fix, landed without a proposal.

Capabilities moved: none written yet (request paths have no spec file).
