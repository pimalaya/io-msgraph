---
cairn: spec
capability: events-ical
status: current
---

# Events iCalendar projection

The `ical` projection of a Graph event onto iCalendar and back. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/rest/users/events/ical.rs header holding the rest.

### Requirement: A stashed component reads back whole
A component the projection stashes (an alarm Graph's single reminder cannot hold) SHALL read back as its lines in order, from its BEGIN to its END, and each stashed property SHALL be restored on its own, so one that no longer tokenises restores nothing rather than corrupting the event.

### Requirement: The stash holds logical lines
A property the projection stashes SHALL be stored as its logical line, its name, parameters and raw value with no fold, no soft break and no line ending, one line per line feed of the stash value, and a stashed component as its logical lines from its BEGIN to its END. A line read from a document SHALL read back with the name, the parameters and the value it was written with wherever its writer folded it, inside a multi-octet character included, and so SHALL the UID. A restored line SHALL be written whole, the folds it arrived with not being kept.

### Requirement: A stash stored folded reads back whole
A stash value that holds lines in the folded shape of their source SHALL read as the lines they fold: a piece that opens on a space or a tab SHALL continue the piece before it, a blank piece SHALL end a line, and a carriage return ending a piece SHALL be dropped.

### Requirement: A weekday that cannot be read is refused
An RRULE whose BYDAY element is not an optional ordinal followed by a two-letter weekday, text outside ASCII included, SHALL be refused as an unsupported recurrence, whether it comes from a document or from a stash.
