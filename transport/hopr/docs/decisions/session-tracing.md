# Session tracing

Carry the originating span alongside each queued message so routing and encoding work can retain
the producer's context across tasks. Queue capture is feature-controlled; consumer futures restore
the retained span while they run.

`HoprSession` retains a `session` span with `session_id` for its lifetime. Socket construction
and every IO poll enter that span, including polls made by a different task after construction.
Outgoing messages carry their local span through the bounded routing queue into a `transport.send`
root with a local `packet_id`. `routing.resolve` and `transport.encode` inherit that root; the CPU
encoder inherits it through `hopr-utils`. Enable debug logging to
include these spans. Session identifiers belong to the parent span, so nested routing, graph and
encoding events can be correlated without copying the identifier into every event.

The outgoing socket sender is now `HoprMessageSender`. Code that explicitly named
`CrossfireSink<(DestinationRouting, ApplicationDataOut)>` should use that alias. Sending still accepts
the same tuple, and clones share the same bounded capacity. Existing packet pipeline builder callers
can continue supplying plain resolved-message tuples; context-aware streams can supply `WithContext`
envelopes created in the producer's scope. The envelope remains local and does not change packet
serialization. Cover traffic has its own root span.

Incoming packets retain a `transport.receive` root with local `packet_id` and `peer` through CPU
decoding, processing, and the session and acknowledgement queues. The decoder adds `session_id`
once the sender is known. Formatted logs expose the ancestor fields without copying them into
child events; use the packet ID to join decode records emitted before the session is known.
Acknowledgements remain batched by peer. An outgoing batch has causal `follows_from` links to all
contributing packets rather than incorrectly choosing one packet as the parent. Trace exporters
can use these links; text logs identify the batch's peer.

Detailed module spans require a build that retains TRACE, such as a debug build. For that build,
`RUST_LOG=debug,hopr_transport::path=trace,hopr_network_graph=trace,hopr_protocol_session=trace`
enables the detailed spans and their parent context. The workspace retains
`release_max_level_debug`, so `RUST_LOG=trace` cannot restore TRACE spans in a release build.

## Candidate dictionaries

Each validated collection is paired with a fresh process-local `set_id` in the planner cache.
The shared rebuild path allocates the pair once, so lazy fills, background refreshes and explicit
recomputations all use the same allocation rule. Cache hits retain the pair through its `Arc`.
IDs distinguish snapshots across planners; they are not cache versions or freshness checks, and
reset on process restart. Pointer equality still decides whether a background refresh is stale.

The candidate records map `(set_id, index)` to a path and its weight. TRACE `candidate selected`
events reference that pair for actual forward and return draws, avoiding a separate copy of the
path in those events. Capture dictionary records from the rebuild as well as subsequent draws;
if logging starts after a cache fill, wait for a refresh to obtain its dictionary.

## Queue context and alternatives

Each queued message carries a local span because the producer and consumer can run in different
futures. Instrumenting only the consumer future, or inspecting the stream after dequeue, cannot
recover the originating session once messages from several sessions share a queue. The sender
captures that context at enqueue time; consumers instrument the work with the retained span.

The default-enabled `tracing-context` feature controls producer-span capture in the queue sender.
Consumers that disable default features can omit it to enqueue without capturing the current span;
message types and bounded-channel behavior remain the same. Explicitly supplied context envelopes
and ordinary tracing events remain available independently of automatic capture.

The adapters currently live in this crate because they support its public sender and pipeline
interfaces. Extracting them into `hopr-utils` requires a companion change in its external repository;
this decision does not introduce a new utility crate or change those interfaces.

## Performance evidence

The [release benchmark report](https://github.com/hoprnet/hoprnet/pull/8604#issuecomment-6650925930)
compares INFO-level routing and session writes against master with the same benchmark and features.
It reports 6.04 us for selector searches, 475 ns for cached forward routes, 992 ns for cached forward
and two return paths, 92.7 us for rebuilds, and 272 ns for 300-byte session writes. None regressed in
that run. These measurements use a sink for formatted output; they do not measure disk or network
logging costs. DEBUG diagnostics add work, and these results do not establish zero overhead for
all workloads or isolate the queue adapter's cost. The report predates restoration of the original
`try_get_with` cache implementation and is not a measurement of the current branch.
