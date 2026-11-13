---
cairn: change
id: projection-and-path-corrections
status: landed
created: 2026-11-13
---

# Projection and path corrections

## Why

The family audit of 2026-11-13 found places where the projections say something else to Graph than the document says, and one refusal the changelog promised that the code did not make.

- **UNTIL** (MSGRAPH-04): an RRULE's UNTIL was cut to its first eight digits, the UTC date, while the read side writes the end of the day in the series zone, told in UTC. Away from UTC the two dates differ: a series at 20:00 in New York ending on 31 December reads back as `UNTIL=20270101T045959Z` and was sent again as `endDate` 1 January, one occurrence too many; east of UTC a last occurrence was lost.
- **Approximated rules** (MSGRAPH-05): the module header promises that a rule Graph cannot hold is refused, yet a BYMONTH or a BYMONTHDAY was ignored on the kinds that have no field for it, each numbered BYDAY element overwrote the index of the one before, and a yearly rule without a BYMONTH took the month of its start. The RRULE is consumed, so the original was gone after the next read.
- **No end** (MSGRAPH-14): a VEVENT with neither a DTEND nor a DURATION was refused, where RFC 5545 3.6.1 gives it one day (a date start) or no time (a date-time start).
- **Rewritten card lines** (MSGRAPH-13): the suffixes of N were dropped, Graph's `generation` not being modelled, and a TEL that was neither `cell` nor `home` filled a business phone slot, a fax or a pager among them.
- **Cut UID** (found while fixing the stash): `MsgraphContact::stashed_uid` read the first value of the UID line, so `UID:a,b;c` answered `a` while `to_vcard` wrote it whole.
- **User id** (MSGRAPH-19): `user_path` encoded a user id but refused nothing, so `..` built `users/../messages/{id}`, which URL resolution turns into another resource. The changelog said it was refused.

## What

- A UTC UNTIL is sent as the date it falls on in the zone of the series start; a date or a local date-time as its date.
- `recurrence` refuses what Graph would only approximate: a BYMONTH or a BYMONTHDAY on a kind without the field, a yearly BYMONTHDAY or BYDAY without a BYMONTH, more than one numbered BYDAY element, numbered beside bare, numbered beside a BYSETPOS, a weekday named twice. Every spelling `rrule` writes still reads back.
- A VEVENT without an end takes the one its start implies. A DTEND or a DURATION that cannot be read is still `NoEnd`, whose message says so.
- `MsgraphContact::generation` is added and mapped onto the N suffixes both ways.
- A TEL typed `fax` or `pager` is stashed. `type_values` reads a quoted TYPE list and the bare parameters of vCard 2.1.
- `stashed_uid` answers the whole raw value.
- `user_path` returns a `Result` through `path_segment`; the 63 coroutines that call it propagate the refusal.

## Not done

MSGRAPH-13 asks for more than this: a consumed line whose parameters or structure the slot cannot hold (an EMAIL with TYPE and PREF, an ADR with a post office box, two NOTE lines, an untyped TEL filed as a business phone) should be kept in the stash and restored while Graph's value agrees, as the To Do projection does. The stash of a card already holds lines of the same names for another reason, the ones past Graph's slots, so a written form and an overflow line cannot be told apart without a marker, and a written form Graph no longer agrees with would come back as a second value. How to mark them, and whether an untyped TEL should leave the business slot at all, is the maintainer's call.
