---
cairn: change
id: stash-logical-lines
status: landed
created: 2026-11-13
---

# The stash holds logical lines

## Why

The three projections (events and tasks onto iCalendar, contacts onto vCard) keep every line Graph has no field for in a single-value extended property, the stash, one line per line feed. The writer stored each line in the wire shape it arrived in, folds included, and the reader split the value at every line feed and restored each piece alone. A line folded by its writer therefore came back as its first physical line only, with a stray carriage return in its value, the rest dropped without an error: silent loss on an ordinary read after a write. A UID over 71 octets was cut the same way, so an event, a task or a contact came back under another UID than the one it was written with, an Exchange `iCalUId` of 112 hex digits included (family audit of 2026-11-13, MSGRAPH-01).

The same audit found a panic in the RRULE reader both iCalendar projections share: a BYDAY element whose byte two from its end sits inside a multi-byte character was split off a character boundary (MSGRAPH-02). The document comes from the other side of a sync, and the consumers build with `panic = "abort"`.

## What

- The stash stores the logical line: the name, the parameters and the raw value, with no fold, no soft break and no ending. A stashed component is its logical lines from its BEGIN to its END. The folds a line arrived with are not kept: a restored line is written whole.
- The reader takes both shapes. A piece that opens on a space or a tab continues the piece before it, and a carriage return ending a piece is dropped, which is how a value stored before this change reads. It is one function for the three projections, `MsgraphSingleValueExtendedProperty::stash_lines`, crate-private, behind `ical` or `vcard`.
- A BYDAY element that cannot be split into an ordinal and a two-letter weekday is refused as an unsupported recurrence.

Out of scope: who owns the stash (this crate or the vCard and iCalendar ones) is a pending decision, so the code stays where it was. A UID an earlier write already stored cut cannot be recovered. A stash stored before this change whose line was broken by a QUOTED-PRINTABLE soft break rather than a fold still loses the pieces after the first. No fuzz targets over the three readers yet.
