---
cairn: spec
capability: contacts-vcard
status: current
---

# Contacts vCard projection

The `vcard` projection of a Graph contact onto vCard and back. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/rest/users/contacts/vcard.rs header holding the rest.

### Requirement: The stash holds logical lines
A line the projection does not map SHALL be stashed as its logical line, its name with its group, its parameters and its whole raw value with no fold, no soft break and no line ending, one line per line feed of the stash value, and SHALL read back with the name, the parameters and the value it was written with wherever its writer folded it, the UID included. A restored line SHALL be written whole, the folds it arrived with not being kept. A stash value that holds lines in the folded shape of their source SHALL read as the lines they fold: a piece that opens on a space or a tab SHALL continue the piece before it, and a carriage return ending a piece SHALL be dropped.

### Requirement: The name suffix rides the generation
The suffixes of N SHALL be written to Graph's `generation` and read back from it, and SHALL clear it when the card holds none.

### Requirement: A fax or a pager number is stashed
A TEL typed `fax` or `pager` SHALL be stashed rather than fill a phone slot, whatever its other types. The types of a line SHALL be read from its TYPE parameters, a quoted list included, and from its bare vCard 2.1 parameters.

### Requirement: The stashed UID is the whole value
`stashed_uid` SHALL answer the whole raw value of the stashed UID line, the commas and semicolons it holds included.

### Requirement: A renaming patch carries the display name
`update_from_vcard` SHALL keep the card's display name in the patch, unchanged as it is, whenever the patch changes `givenName`, `middleName`, `surname`, `title`, `generation` or `companyName`: Graph recomposes `displayName` from the name parts on a PATCH changing one of them that names no `displayName`, overwriting the card's own. A patch changing none of them SHALL leave an unchanged display name out.

#### Scenario: An organization changed on a contact named apart from its parts
- GIVEN a base card `FN:Jane the Pilot`, `N:Doe;Jane;;;`, and the same card with `ORG:Acme` added
- WHEN `update_from_vcard` projects the edit
- THEN the patch sets `companyName` and `displayName: "Jane the Pilot"`
