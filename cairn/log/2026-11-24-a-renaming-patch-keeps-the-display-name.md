---
cairn: log
change: a-renaming-patch-keeps-the-display-name
landed: 2026-11-24
---

# A renaming patch keeps the display name

Found by the second live night (workspace .lanes/live/journal-2.md, the cardamum lines). `update_from_vcard` left every unchanged field out of the PATCH, `displayName` included, and Graph recomposes `displayName` from the name parts on a PATCH that changes one of them or `companyName` without naming it: raw probes on the test account rewrote a set display name on `givenName`, `middleName`, `surname`, `title`, `generation` (set and cleared) and `companyName`, and kept it on `jobTitle`, `department`, `personalNotes`, `mobilePhone`, `nickName` and `fileAs`. So `cardamum card update --organization` on Graph turned `FN:live-2026-11-24-cardamum-ms2` into `FN:live ms2` without a word, and a neverest push of a store edit did the same while the store kept the old FN, the next sync saying "already in sync".

Such a patch now carries the card's display name. Pinned by `update_from_vcard_of_a_name_part_keeps_the_display_name`, which failed on `Unset` before the fix; `a_name_suffix_rides_the_generation` now expects `displayName` beside the cleared `generation`. Live with cardamum built against this tree: `--organization OrgZ` and `--family-name ms2c` both kept `FN:live-2026-11-24-cardamum-ms2`.

Capabilities moved: contacts-vcard (a renaming patch carries the display name).
