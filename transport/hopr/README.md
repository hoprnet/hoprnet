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

## Routing diagnostics

DEBUG records summarize graph traversal, selector filtering, validation, and successful candidate-set
installations. Counts describe the candidates returned within the search limit, not all possible paths.
TRACE records add candidate weights, effective return-path probabilities, cache loads, and actual path
choices. A process-local `set_id` connects those choices to the immutable candidate set installed in
the cache, including choices made on later cache hits.

Detailed routing diagnostics require a build that retains TRACE, such as a debug build. For that
build, `RUST_LOG=debug,hopr_transport::path=trace,hopr_network_graph=trace` enables the detailed events
and their parent context. The workspace intentionally retains `release_max_level_debug`: production
release builds provide DEBUG summaries, while candidate and per-draw TRACE events are compiled out.
Setting `RUST_LOG=trace` alone cannot restore those events in a release build. Changing the production
compile-time ceiling is a separate logging-policy change.
