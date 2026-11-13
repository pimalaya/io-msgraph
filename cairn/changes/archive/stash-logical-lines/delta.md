---
cairn: delta
change: stash-logical-lines
---

# Delta

## ADDED Requirements

### Requirement: The stash holds logical lines (events-ical)
A property the projection stashes SHALL be stored as its logical line, its name, parameters and raw value with no fold, no soft break and no line ending, one line per line feed of the stash value, and a stashed component as its logical lines from its BEGIN to its END. A line read from a document SHALL read back with the name, the parameters and the value it was written with wherever its writer folded it, inside a multi-octet character included, and so SHALL the UID. A restored line SHALL be written whole, the folds it arrived with not being kept.

### Requirement: A stash stored folded reads back whole (events-ical)
A stash value that holds lines in the folded shape of their source SHALL read as the lines they fold: a piece that opens on a space or a tab SHALL continue the piece before it, a blank piece SHALL end a line, and a carriage return ending a piece SHALL be dropped.

### Requirement: A weekday that cannot be read is refused (events-ical)
An RRULE whose BYDAY element is not an optional ordinal followed by a two-letter weekday, text outside ASCII included, SHALL be refused as an unsupported recurrence, whether it comes from a document or from a stash.

### Requirement: The stash holds logical lines (contacts-vcard)
A line the projection does not map SHALL be stashed as its logical line, its name with its group, its parameters and its whole raw value with no fold, no soft break and no line ending, one line per line feed of the stash value, and SHALL read back with the name, the parameters and the value it was written with wherever its writer folded it, the UID included. A restored line SHALL be written whole, the folds it arrived with not being kept. A stash value that holds lines in the folded shape of their source SHALL read as the lines they fold: a piece that opens on a space or a tab SHALL continue the piece before it, and a carriage return ending a piece SHALL be dropped.

## MODIFIED Requirements

### Requirement: Every line without a Graph slot survives (todo)
A line the projection does not map SHALL be stashed as its logical line, as events stash theirs, in the single-value extended property events use and restored on read, the UID among them, a stashed component whole, and a stash stored folded SHALL read back whole as it does for events. A managed line Graph holds coarser than written (a DUE with a time, a COMPLETED, a PRIORITY other than 1 and 9, a STATUS other than IN-PROCESS and COMPLETED, an RRULE spelled otherwise) SHALL be stashed too and restored in place of the derived line while Graph's value agrees with it, and dropped once it no longer does. An RRULE written without a DUE or a DTSTART to anchor it SHALL be stashed and restored while Graph holds no date and no recurrence.

## REMOVED Requirements

None.
