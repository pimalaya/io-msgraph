---
cairn: spec
capability: paging
status: current
---

# Paging

The page a listing or a delta round answers, and how the links it carries are followed. This file grows by accretion: it states the requirements changes have touched so far, the src/v1/page.rs header holding the rest.

### Requirement: Every listing answers one page type
Every listing and every delta round SHALL answer the same page type: its items, the next link, the delta link and the count. A serialised page SHALL leave out a delta link and a count it does not hold.

### Requirement: A link is followed on the Graph origin only
A coroutine built from a link, the page coroutine and every `from_link`, SHALL send the link as it is, and SHALL refuse with `InvalidRequest`, before anything is sent, a link whose scheme, host or port is not that of the API base.
