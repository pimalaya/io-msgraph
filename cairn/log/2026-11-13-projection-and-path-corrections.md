---
cairn: log
change: projection-and-path-corrections
landed: 2026-11-13
---

# Projection and path corrections

Second batch from the family audit of 2026-11-13: MSGRAPH-04, MSGRAPH-05, MSGRAPH-14, MSGRAPH-19, part of MSGRAPH-13, and a cut UID found while fixing the stash.

**UNTIL** (MSGRAPH-04). `recurrence` took the first eight digits of UNTIL, while `until` writes the end of the last day in the recurrence zone as a UTC stamp. A UTC stamp is now told in the zone of the series start (`zone`, so a Windows name resolves too) before its date is taken; a zone neither table knows keeps the UTC date, as `until` writes it. The wrong date only left on a create or on a change of the rule, both sides of an update going through the same cut. Test `an_until_ends_on_its_date_in_the_series_zone`, which showed `2027-01-01` for a New York series ending on `2026-12-31`.

**Approximated rules** (MSGRAPH-05). The numbered BYDAY elements are now collected apart from BYSETPOS: one alone is the index, anything else beside it is refused, since Graph counts one index over all its days (`BYDAY=2TU,2WE` is two days a month, Graph's second of Tuesday and Wednesday is one). The day of month and the month are matched against the kind, so a part the kind has no field for is refused instead of ignored, and a yearly rule without a BYMONTH is taken only when it repeats on the date of its start. Beyond the audit's list, a yearly BYDAY without a BYMONTH is refused too: RFC 5545 3.3.10 counts it over the year, Graph over the month of the start. Tests `a_rule_graph_would_only_approximate_is_refused` (it showed `FREQ=DAILY;BYMONTH=1` taken) and `a_rule_graph_holds_exactly_is_taken`, a guard that never failed: it holds the exact shapes and the six spellings `rrule` writes, so a document read from Graph still writes back. The task projection shares the reader.

**No end** (MSGRAPH-14). A VEVENT with neither line now ends one day after a date start and at its start otherwise. DURATION is tracked as seen or not, so one that does not parse is still `NoEnd`, as is a floating DTEND; the message of `NoEnd` was reworded to match. Whether Graph takes an event that ends as it starts was not checked live. Test `an_event_without_an_end_takes_the_one_its_start_implies`, which showed `NoEnd`.

**Card lines** (MSGRAPH-13, in part). `MsgraphContact::generation` is new and carries the N suffixes both ways. Graph's contact resource names it as the suffix, which was not checked against the service for this change: the live vCard round trip now writes a suffix, so the next run of the suite checks it. A TEL typed `fax` or `pager` goes to the stash before the cell and home tests. `type_values` splits a quoted TYPE list (`TYPE="voice,home"`, the form of RFC 6350's own example) and takes the bare parameters of vCard 2.1 (`TEL;WORK;FAX`), which also files a `TEL;CELL` as the mobile phone where it used to fill a business slot. Tests `a_name_suffix_rides_the_generation` (`generation` came back Unset) and `a_fax_or_a_pager_is_stashed_rather_than_filed_as_a_phone` (the business phones held `+331` and `+332`). The fixture `full_contact` and the N line `to_vcard_projects_every_mapped_field` expects gained the suffix, the projection managing one more field.

**Cut UID.** `stashed_uid` of a contact read `raw_value_str`, which vcard-rs documents as the first value. It now reads the value node whole. Test `a_uid_holding_separators_is_answered_whole` (`urn:x:a` for `urn:x:a,b;c`).

**User id** (MSGRAPH-19). `user_path` goes through `path_segment` and returns its refusal; `me` is answered before. Every constructor that builds a path from a user id already returned `MsgraphSendError`, so the 63 call sites gained a `?` and no signature but `user_path` moved. Test `rejects_a_user_id_url_resolution_drops`, which showed an empty user id accepted.

Left open:

- The rest of MSGRAPH-13: a consumed EMAIL, ADR or NOTE still reads back in Graph's shape, and an untyped TEL still fills a business slot. Keeping the written form needs a way to tell it from the overflow lines the stash already holds under the same names.
- `generation` and a zero-length event await a run of the live suite.
- An event already on Graph under a rule now refused reads as before. Writing it again needs a document whose rule Graph can hold, which the one read back is.

Capabilities moved: events-ical (three requirements added), contacts-vcard (three added), request-paths (new file, by accretion).
