---
cairn: tasks
change: stash-logical-lines
---

- [x] Reproduce in the crate: a folded unmanaged line, a folded 112-character UID and a stash stored folded, in the three projections; a multi-byte BYDAY element in the two iCalendar ones
- [x] Write the logical line to the stash (events, tasks, contacts), a stashed component as its logical lines
- [x] Read both shapes through one `MsgraphSingleValueExtendedProperty::stash_lines`
- [x] Refuse a BYDAY element that does not split on a character boundary
- [x] Round-trip sweep per projection: three line heads, values of 0 to 160 characters of one to four octets, eight fold widths, CRLF and LF
- [x] CHANGELOG.md, fold into cairn/spec/events-ical.md, cairn/spec/todo.md and cairn/spec/contacts-vcard.md, log, archive
- [ ] Fuzz targets over `MsgraphEvent::from_ical`, `MsgraphTodoTask::from_ical` and `MsgraphContact::from_vcard` (not done: the crate has no fuzz layout)
