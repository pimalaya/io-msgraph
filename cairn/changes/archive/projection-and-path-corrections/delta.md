---
cairn: delta
change: projection-and-path-corrections
---

# Delta

## ADDED Requirements

### Requirement: An UNTIL ends the series on its date in the series zone (events-ical)
An RRULE's UNTIL given as a UTC date-time SHALL be sent as the `endDate` it falls on in the zone of the series start, a Windows zone name read as its IANA counterpart, and one given as a date or as a local date-time SHALL be sent as its date. An `endDate` read from Graph and written again SHALL be the same date.

### Requirement: A rule Graph would only approximate is refused (events-ical)
An RRULE SHALL be refused as an unsupported recurrence when Graph's pattern would not repeat on the same days: a BYMONTH or a BYMONTHDAY on a kind with no field for it (daily, weekly, relative), a yearly BYMONTHDAY without a BYMONTH, a yearly BYDAY without a BYMONTH, more than one numbered BYDAY element, a numbered element beside a bare one or beside a BYSETPOS, and a weekday named twice. Every rule the projection writes SHALL read back as written.

### Requirement: An event without an end takes the one its start implies (events-ical)
A VEVENT with neither a DTEND nor a DURATION SHALL end one day after a date start and at its start for a date-time one (RFC 5545 3.6.1). A DTEND or a DURATION that cannot be read SHALL still be refused.

### Requirement: The name suffix rides the generation (contacts-vcard)
The suffixes of N SHALL be written to Graph's `generation` and read back from it, and SHALL clear it when the card holds none.

### Requirement: A fax or a pager number is stashed (contacts-vcard)
A TEL typed `fax` or `pager` SHALL be stashed rather than fill a phone slot, whatever its other types. The types of a line SHALL be read from its TYPE parameters, a quoted list included, and from its bare vCard 2.1 parameters.

### Requirement: The stashed UID is the whole value (contacts-vcard)
`stashed_uid` SHALL answer the whole raw value of the stashed UID line, the commas and semicolons it holds included.

### Requirement: Every id is one path segment (request-paths)
An id spliced into a request path, the user id included, SHALL be percent-encoded outside the RFC 3986 `pchar` set, and SHALL be refused with `InvalidRequest` before anything is sent when it is empty, `.` or `..`. The user id `me` SHALL address the authenticated user as it is.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
