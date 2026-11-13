---
cairn: log
change: stash-logical-lines
landed: 2026-11-13
---

# The stash holds logical lines

Found by the family audit of 2026-11-13 (MSGRAPH-01, critical, and MSGRAPH-02, high).

The stash of the three projections stored each unmanaged line as `Display` writes it, which is the wire shape it arrived in, folds included, joined the lines with a line feed and split the value at every line feed on read. A line folded by its writer was therefore two or more entries: the first restored cut, with a stray carriage return in its value, the others refused by `push_raw` for want of a colon and dropped by `let _ =`. `stashed_uid` answered the first 71 octets of a longer UID, so an event Exchange itself minted (a 112-digit `iCalUId`, which `to_ical` writes folded) read back under another UID from the first write that sent the stash. Nothing reported it: the guard the consumers run compares two values cut the same way.

The writer now builds the logical line from its parts: the name, each parameter and the raw value (`raw_line` of src/v1/rest/users/events/ical.rs, shared with the tasks; inline in src/v1/rest/users/contacts/vcard.rs, where the value node's `Display` is the whole value, `raw_value_str` being its first value only). `raw_component` walks the component instead of splitting its text. The reader is one function for the three, `MsgraphSingleValueExtendedProperty::stash_lines` in src/v1/rest/users/contacts.rs, behind `any(feature = "ical", feature = "vcard")`: it re-joins a piece that opens on a space or a tab onto the piece before it and drops a carriage return ending a piece, so a stash stored at the provider in the old shape reads whole again without being rewritten. The new shape is the one the old reader already took for a line that arrived unfolded, so an older build still reads a stash written by this one.

The RRULE reader split a BYDAY element two bytes from its end with `split_at`, which panics off a character boundary (`BYDAY=€`). It now uses `split_at_checked` and refuses the rule as `UnsupportedRecurrence`. The task projection reached the same code from a document and from a stashed RRULE.

Tests, all failing on the previous code: `an_unmanaged_line_round_trips_whatever_its_folds` in the three projections (three line heads, a quoted parameter outside ASCII and a grouped name among them, values of 0 to 160 characters of one to four octets, eight fold widths from 1 to none, CRLF and LF; it showed `["X\r", " -\r", "\tC\r", ...]` where `["X-CUSTOM:"]` was expected), `a_fold_inside_a_character_is_stashed_whole` (events), `a_folded_uid_reads_back_whole` (events, contacts) and `a_folded_uid_and_a_folded_written_form_read_back_whole` (tasks), which showed the UID cut at 71 characters, `a_stash_stored_folded_reads_back_whole` in the three (the old shape, swept for events), `a_stashed_alarm_keeps_its_folded_lines` (events) and `a_weekday_that_is_not_ascii_is_refused` (events, tasks: `end byte index 1 is not a char boundary`). The test helpers the three modules share sit in the `tests` module of src/v1/rest/users/contacts.rs.

Left open:

- A restored line is written whole, however long, where RFC 5545 3.1 and RFC 6350 3.2 ask for folds at 75 octets (a SHOULD). That was already so for a line that arrived unfolded.
- A stash stored before this change whose line was broken by a QUOTED-PRINTABLE soft break still loses the pieces after the first. New writes store it joined.
- A UID an earlier write already stored cut stays cut.
- `MsgraphContact::stashed_uid` reads the first value of the UID line, so a UID holding an unescaped comma or semicolon is answered cut there, while `to_vcard` writes it whole.
- The consumers' guard should compare with the UID of the document, not with `stashed_uid` of the outgoing body.
- No fuzz target over `from_ical` and `from_vcard`: the crate has no fuzz layout yet.
- Who owns the stash, this crate or the vCard and iCalendar ones, is undecided; nothing moved.

Capabilities moved: events-ical ("The stash holds logical lines", "A stash stored folded reads back whole", "A weekday that cannot be read is refused"), todo ("Every line without a Graph slot survives"), contacts-vcard (new file, by accretion).
