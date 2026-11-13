---
cairn: spec
capability: errors
status: current
---

# API errors

What an exchange tells its caller beside the parsed body: the error of a non-2xx answer and the headers of a 2xx one. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/send.rs header holding the rest.

### Requirement: An API error carries what the answer asks of the caller
A non-2xx answer SHALL complete as an API error holding the status, the code and the message of Graph's error envelope, the method of the request, the delay of a `Retry-After` header given in seconds (a date reading as none), the `WWW-Authenticate` header, the request id (the `request-id` header, else the one of the envelope) and whether the connection can be reused. An error read from a batch item SHALL hold the headers of that item and no method, and SHALL read the connection as reusable.

### Requirement: A request is retryable only where a repeat is safe
A throttled answer (429) SHALL read as retryable whatever the method. A 500, 502, 503 or 504 SHALL read as retryable for a GET, a HEAD, a PUT or a DELETE only, a POST or a PATCH possibly having been applied.

### Requirement: A request that cannot be serialised is refused unsent
A request whose method or header name is not a token, or whose header value holds a CR, a LF or a NUL, the access token included, SHALL complete its first resume with `InvalidRequest`, nothing written, and the refusal SHALL NOT hold the header value.

### Requirement: A response carries its headers
A 2xx answer SHALL complete with the response headers, in order, names lowercased, beside the parsed body and the connection reuse flag.
