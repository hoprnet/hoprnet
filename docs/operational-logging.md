# Operational logging

A configured Hopr node starts one managed periodic state reporter after account
readiness. `HOPR_STATE_DUMP_INTERVAL` is integer seconds: default 60, `0` disables
reporting and producer counters, and enabled values must be 5–86400. Invalid values
fail startup with a configuration error. The task is owned by the node's abortable
process list and is cancelled with the other node tasks.

Each interval, including intervals without application traffic, emits an INFO event
with `event=node_state` and a JSON `state` string. With a JSON tracing subscriber:

```sh
jq 'select(.fields.event == "node_state") | .fields.state | fromjson' node.jsonl
```

The record contains exact Safe balance/allowance, source-visible local channel and
funding summaries, connected/announced/open-channel peer counts, destination and
search-phase health, sibling forward/reply SURB counts, and neighbor/loopback probe
outcomes. The connector caps the JSON payload at 32 KiB, excluding the logger
wrapper. It reads adjacent local channels in batches and preserves partial results,
read errors, timeouts, omitted detail and counter overflow. It is not an atomic or
freshness-verified chain snapshot. Identities use the connector's JSON serialization;
compare serialized identities rather than assuming a particular display format.

The transport does not depend on the state dump. It reports `TransportEvent`s (path
resolves, graph searches, SURB mints and returns, probe outcomes) to an optional
`TransportObserver`, and `hopr_lib::state_dump` records them in the connector's
`IntervalCounters`. Successful resolves and SURB counts never wait for the counters'
lock; when it is busy they are counted in the totals and overflow only. Resolver
calls include cache hits; selector counters count actual queries and their phase
results. Return-path batches count once per batch. Known destinations normalize to
offchain keys; an unresolved chain address remains a chain identity. SURB producers
record mint/return events independently of the graph's telemetry drain. Returns may
cross interval boundaries, so minted/returned counts are not delivery probabilities.
Probe counters register when probing starts and count matched replies or actual
expiries; unknown/late replies do not count a second time. Unavailable inputs remain
distinct from genuine zero activity.

Custom chain implementations used with the node builder must implement the
connector's error-preserving `StateRead` diagnostic interface. Custom transport
embedders can pass their own observer to `HoprTransport::new_with_observer`;
`HoprTransport::new` reports nothing.

The embedding node can add up to four named sections to every record with
`HoprBuilderConfigured::with_state_dump_section`, for example the states of its
strategies (`MultiStrategy::state_observer` in hopr-strategy). Each section is read
once per record and capped at 2 KiB. Underfunded outgoing channels are listed by name
(up to sixteen, lowest balance first); incoming channels are funded by their
counterparty and are not counted as underfunded.

Routine probe ping/pong, expiry, path resolution, selector candidates/search failures,
return-path draws, SURB mint/decode and per-pair round-trip details use TRACE. Relayer
sets used only for path diagnostics are also computed only when TRACE is enabled.
DEBUG retains aggregate cache updates and actionable degradation transitions. SURB
flushes retain one aggregate INFO event per active interval. Named blind-spot details
are capped at eight destinations, with an omitted count. TRACE requires a build that
retains that static level.

Connector and graph crates are sourced from the hopr-impls CORE-1168 Git branch;
Cargo.lock pins the resolved revision. No sibling checkout is needed.

The `state_dump` integration test runs enabled/disabled configurations in isolated
processes. It verifies real INFO records from an emulated node cluster with channel
inventory, a routed echo session, routing/search/SURB/probe activity, continuing
periodic emission, and the payload budget. This complements unit tests for cache
hits, failed searches, probe expiry and independent SURB counter windows. Live
production CPU and log-volume measurements remain rollout validation.
