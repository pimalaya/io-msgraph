---
cairn: spec
capability: messages
status: current
---

# Messages

The mail message surface (`users/{id}/messages`, Graph's `message`), as typed coroutines over `MsgraphMessage`. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/rest/users/messages.rs header holding the rest.

### Requirement: A message carries its Internet header fields as returned
A message SHALL carry the `internetMessageHeaders` Graph returns as name and value pairs in the order returned, a repeated name kept once per occurrence and no value interpreted, and SHALL carry an empty list when Graph returns none, leaving the member out or answering `null` for it, so one message never fails the page it is listed on. An empty list SHALL be left out of a written body.

### Requirement: The caller selects what a message read returns
A listing, a delta round and a get SHALL send the `$select` and the `$expand` the caller passes and none of their own, so a property Graph returns only selected or expanded (the Internet header fields, the extended properties) is asked for by naming it there.

### Requirement: An update can clear the recipients
The To, Cc, Bcc and Reply-To lists of a message SHALL be tri-state: left out of a body when unset, so an update keeps what Graph holds, and sent as an empty list when set empty, so an update clears them.

### Requirement: An attachment over what one request takes is refused
An attachment create whose content is over 3 MiB SHALL be refused with `InvalidRequest` before anything is sent, Graph taking a larger file through an upload session the crate does not model.
