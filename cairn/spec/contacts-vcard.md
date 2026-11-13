---
cairn: spec
capability: contacts-vcard
status: current
---

# Contacts vCard projection

The `vcard` projection of a Graph contact onto vCard and back. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/rest/users/contacts/vcard.rs header holding the rest.

### Requirement: The stash holds logical lines
A line the projection does not map SHALL be stashed as its logical line, its name with its group, its parameters and its whole raw value with no fold, no soft break and no line ending, one line per line feed of the stash value, and SHALL read back with the name, the parameters and the value it was written with wherever its writer folded it, the UID included. A restored line SHALL be written whole, the folds it arrived with not being kept. A stash value that holds lines in the folded shape of their source SHALL read as the lines they fold: a piece that opens on a space or a tab SHALL continue the piece before it, and a carriage return ending a piece SHALL be dropped.
