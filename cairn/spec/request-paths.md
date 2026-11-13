---
cairn: spec
capability: request-paths
status: current
---

# Request paths

How the coroutines build the path of a request from the ids they are given. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/send.rs header holding the rest.

### Requirement: Every id is one path segment
An id spliced into a request path, the user id included, SHALL be percent-encoded outside the RFC 3986 `pchar` set, and SHALL be refused with `InvalidRequest` before anything is sent when it is empty, `.` or `..`. The user id `me` SHALL address the authenticated user as it is.
