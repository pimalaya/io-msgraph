---
cairn: log
change: message-categories-tri-state
landed: 2026-11-12
---

# A message update can clear the categories

`MsgraphMessage::categories` was a `Vec<String>` skipped when empty, so a PATCH could never send `[]` and no update could remove the last category: Himalaya's `flag set` left every category on the message. It is now a `MsgraphField<Vec<String>>`, as the contact collections are: `Unset` leaves the categories alone, `Set(vec![])` clears them, and a read message carries `Set` or `Unset`. Its JSON schema reads as an optional list.

Test `an_update_can_clear_the_categories`, which does not compile against the old type and whose assertion the old serialization fails. Breaking change of a public field type, landed without a proposal at the maintainer's campaign request.

Capabilities moved: none written yet (messages have no spec file).
