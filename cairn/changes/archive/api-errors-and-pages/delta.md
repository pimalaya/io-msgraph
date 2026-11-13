---
cairn: delta
change: api-errors-and-pages
---

# Delta

## ADDED Requirements

### Requirement: An API error carries what the answer asks of the caller (errors)
A non-2xx answer SHALL complete as an API error holding the status, the code and the message of Graph's error envelope, the method of the request, the delay of a `Retry-After` header given in seconds (a date reading as none), the `WWW-Authenticate` header, the request id (the `request-id` header, else the one of the envelope) and whether the connection can be reused. An error read from a batch item SHALL hold the headers of that item and no method, and SHALL read the connection as reusable.

### Requirement: A request is retryable only where a repeat is safe (errors)
A throttled answer (429) SHALL read as retryable whatever the method. A 500, 502, 503 or 504 SHALL read as retryable for a GET, a HEAD, a PUT or a DELETE only, a POST or a PATCH possibly having been applied.

### Requirement: A response carries its headers (errors)
A 2xx answer SHALL complete with the response headers, in order, names lowercased, beside the parsed body and the connection reuse flag.

### Requirement: Every listing answers one page type (paging)
Every listing and every delta round SHALL answer the same page type: its items, the next link, the delta link and the count. A serialised page SHALL leave out a delta link and a count it does not hold.

### Requirement: A link is followed on the Graph origin only (paging)
A coroutine built from a link, the page coroutine and every `from_link`, SHALL send the link as it is, and SHALL refuse with `InvalidRequest`, before anything is sent, a link whose scheme, host or port is not that of the API base.

### Requirement: An update can clear the recipients (messages)
The To, Cc, Bcc and Reply-To lists of a message SHALL be tri-state: left out of a body when unset, so an update keeps what Graph holds, and sent as an empty list when set empty, so an update clears them.

### Requirement: An attachment over what one request takes is refused (messages)
An attachment create whose content is over 3 MiB SHALL be refused with `InvalidRequest` before anything is sent, Graph taking a larger file through an upload session the crate does not model.

### Requirement: An HTML body reads as the text it shows (events-ical)
The text of an HTML body SHALL leave out what a head, a style, a script and a comment hold, SHALL break the line at a `br` and at the end of a `p` or a `div`, and SHALL read character references in one pass: the named `amp`, `lt`, `gt`, `quot`, `apos` and `nbsp`, and the numeric ones, a no-break space reading as a space.

## MODIFIED Requirements

None.

## REMOVED Requirements

None.
