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

### Requirement: An UNTIL ends the series on its date in the series zone
An RRULE's UNTIL given as a UTC date-time SHALL be sent as the `endDate` it falls on in the zone of the series start, a Windows zone name read as its IANA counterpart, and one given as a date or as a local date-time SHALL be sent as its date. An `endDate` read from Graph and written again SHALL be the same date.

### Requirement: A rule Graph would only approximate is refused
An RRULE SHALL be refused as an unsupported recurrence when Graph's pattern would not repeat on the same days: a BYMONTH or a BYMONTHDAY on a kind with no field for it (daily, weekly, relative), a yearly BYMONTHDAY without a BYMONTH, a yearly BYDAY without a BYMONTH, more than one numbered BYDAY element, a numbered element beside a bare one or beside a BYSETPOS, and a weekday named twice. Every rule the projection writes SHALL read back as written.

### Requirement: An event without an end takes the one its start implies
A VEVENT with neither a DTEND nor a DURATION SHALL end one day after a date start and at its start for a date-time one (RFC 5545 3.6.1). A DTEND or a DURATION that cannot be read SHALL still be refused.

### Requirement: An HTML body reads as the text it shows
The text of an HTML body SHALL leave out what a head, a style, a script and a comment hold, SHALL break the line at a `br` and at the end of a `p` or a `div`, and SHALL read character references in one pass: the named `amp`, `lt`, `gt`, `quot`, `apos` and `nbsp`, and the numeric ones, a no-break space reading as a space.
