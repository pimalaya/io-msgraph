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

### Requirement: A series' instances are read in windows Graph accepts
With the `ical` feature, a series' recurrence SHALL give the windows its instances are read in within a range the caller names: the series' bounds, an open-ended series ending where the range does, intersected with that range, its last day included, cut into windows of five years less a day, each as the RFC 3339 UTC instants `event_instances` takes. Graph refuses an `instances` listing over more than five years, so no window SHALL exceed it. A series the range does not meet SHALL give no window, and one whose bounds do not read none at all.

#### Scenario: An open-ended series older than five years
- GIVEN a weekly series starting 2019-01-07 with no end
- WHEN its windows are asked within 2021-11-15 to 2031-11-15
- THEN they start on 2021-11-15, end on 2031-11-15, and none spans five years or more

### Requirement: A series is written with the changes of its instances
With the `ical` feature, a series document SHALL project onto the write of its master and the change of each instance the document sets apart from its rule, made once the master is written: a delete for an occurrence it cancels, by an EXDATE or by an override whose STATUS is CANCELLED, and a patch for an occurrence it overrides, holding the fields the override sets differently from the instance as Graph holds it, never its recurrence nor its stash. For a create, every occurrence SHALL be taken to follow the series, carried to its original start. For an update, the instance SHALL be taken as `base` holds it: its exception, or the series carried there; an occurrence `base` overrides and the document does not SHALL be patched back to the series carried there, and one both cancel SHALL take no change. An EXDATE and an override of the same occurrence SHALL cancel it. An update whose master patch moves the series (its start, end, all-day flag or recurrence) SHALL take the exceptions and cancelled occurrences of `base` as dropped, Graph not keeping them across such a patch, and SHALL change the instances as after a create.

#### Scenario: A series with one EXDATE and one override
- GIVEN a daily series of four from 2026-11-23 14:00 in Europe/Paris, an EXDATE on 2026-11-25 and an override moving 2026-11-24 to 16:00
- WHEN it is projected for a create
- THEN the master is the create body `create_from_ical` gives, and the changes, in the order of their original start, are a patch of the instance originally starting 2026-11-24T13:00:00Z holding its new start and end alone, then a delete of the one originally starting 2026-11-25T13:00:00Z

### Requirement: A change names the instance it touches
Each instance change SHALL carry the occurrence's original start as Graph's `originalStart` tells it, an RFC 3339 UTC instant, or the date for an all-day series, and the window `event_instances` lists it within: from the day before that start, or before the start `base` moved its exception to, to two days after the later of them, at midnight UTC. A listed instance SHALL be the one a change names when its `originalStart` is the same instant, or falls on the same UTC date for an all-day series.

### Requirement: What Graph cannot hold of a series is named
The series projection SHALL drop nothing silently: it SHALL name, as its logical line, every RDATE and EXRULE of the master; every EXDATE value and RECURRENCE-ID whose instant does not read; every RECURRENCE-ID with a RANGE; every override and EXDATE beside a master that is no series; every occurrence `base` cancels and the document no longer does, Graph having no call restoring one; and every unmanaged line of an override other than its UID, an instance patch carrying no stash.

### Requirement: An event without a description is created with no body
A create body (`create_from_ical`, and the master of `MsgraphEventSeries::create_from_ical`) SHALL leave the body out when the document gives no text for it, no DESCRIPTION nor HTML one, or an empty one: Exchange rewrites an event 0.5 to 4 seconds after a write that sets its body, even an empty one, moving its `changeKey`, so a change queued right after the create would meet a revision its writer never made. An update SHALL still clear a removed description with an empty text body, Graph refusing a null one.

#### Scenario: A series created with no DESCRIPTION
- GIVEN a weekly series document with a SUMMARY and no DESCRIPTION
- WHEN it is projected for a create
- THEN the create body, and the series master's, carry no `body` member
