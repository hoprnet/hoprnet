# hopr-transport

Implements the main public interface for the core HOPR library - that is the `Hopr` type.

The application representing the entire physical transport functionality, including:

- p2p
- protocol
- session

## Session tracing

`HoprSession` retains a `session` span with `session_id` for its lifetime. Socket construction
and every IO poll enter that span, including polls made by a different task after construction.
Outgoing messages carry their local span through the bounded routing queue into `routing.resolve`
and `transport.encode`; the CPU encoder inherits it through `hopr-utils`. Enable debug logging to
include these spans. Session identifiers belong to the parent span, so nested routing, graph and
encoding events can be correlated without copying the identifier into every event.

The outgoing socket sender is now `HoprMessageSender`. Code that explicitly named
`CrossfireSink<(DestinationRouting, ApplicationDataOut)>` should use that alias. Sending still accepts
the same tuple, and clones share the same bounded capacity. Existing packet pipeline builder callers
can continue supplying plain resolved-message tuples; context-aware streams can supply `WithContext`
envelopes created in the producer's scope. The envelope remains local and does not change packet
serialization. Cover traffic has its own root span.
