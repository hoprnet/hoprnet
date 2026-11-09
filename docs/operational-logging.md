# Operational logging

With the opt-in `hopr-lib/state-dump` Cargo feature enabled, install a reporter via
`HoprBuilderConfigured::with_state_dump`. The node starts its managed reporting
task after account readiness. The behavior and limits below describe the companion
collector used by the reference adapter. `HOPR_STATE_DUMP_INTERVAL` is integer
seconds: default 60, `0` disables
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

For a network with 1,000 nodes and 50 edges per node, the record remains a bounded
local-node summary; it does not serialize the entire 50,000-edge graph. At the
32 KiB payload limit and the default 60-second interval, one node emits at most
45 MiB/day or about 3.96 GiB over 90 days (129,600 records). Across 1,000 reporting
nodes that is about 3.86 TiB over 90 days. These are payload-only upper bounds;
JSON escaping, logger metadata and log shipping add overhead, while compression
can reduce stored size. A shorter configured interval increases volume accordingly.

The dump lives in the node's normal tracing output, using the application's log
sink. This API adds no file, database, network endpoint, log shipping or retention
policy. Operators choose collection, rotation and retention for those logs. For
example, these are selected fields from the decoded `state` schema; the full record
also includes node identity, timestamps, channel inventory and routing counters:

```json
{
  "schema_version": 1,
  "sequence": 1,
  "safe_balance": { "status": "ok", "value": "1000000000000000000" },
  "safe_allowance": { "status": "ok", "value": "2000000000000000000" },
  "connected_peers": 50,
  "announced_peers": { "status": "ok", "value": 1000 }
}
```

The values above are illustrative, not measurements from a 1,000-node deployment.

The transport does not depend on the state dump. It reports `TransportEvent`s (path
resolves, graph searches, SURB mints and returns, probe outcomes) to an optional
`TransportObserver`. The embedding application supplies a `StateDumpFactory` that
connects these events to its counters and returns a lazy `StateDump` task. The
connector-backed adapter in [the test wiring](../hopr/hopr-lib/src/testing/state_dump.rs)
shows how to use the companion collector and its `IntervalCounters`. Successful
resolves and SURB counts never wait for the counters'
lock; when it is busy they are counted in the totals and overflow only. Resolver
calls include cache hits; selector counters count actual queries and their phase
results. Return-path batches count once per batch. Known destinations normalize to
offchain keys; an unresolved chain address remains a chain identity. SURB producers
record mint/return events independently of the graph's telemetry drain. Returns may
cross interval boundaries, so minted/returned counts are not delivery probabilities.
Probe counters register when probing starts and count matched replies or actual
expiries; unknown/late replies do not count a second time. Unavailable inputs remain
distinct from genuine zero activity.

`hopr-lib` requires only the chain interfaces from `hopr-api`, even with
`state-dump` enabled. The application owns any additional reads its collector
needs; the companion collector uses the connector's error-preserving `StateRead`
interface. Custom transport embedders can pass an observer directly to
`HoprTransport::new_with_observer`; `HoprTransport::new` reports nothing.

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

The `state-dump` feature exposes interfaces only and is disabled by default. It does
not enable a connector implementation or install a reporter automatically. A factory's
`prepare` method validates configuration before component construction; `create`
receives the node's chain, graph, network, Safe address and named sections before
transport construction. It returns the observer and a lazy reporting future, or
`None` to disable both. The node owns spawning and cancellation. The reference test
adapter implements the interval behavior described above; applications can supply
their own reporter without implementing any connector-specific chain trait.

The companion connector and graph remain test dependencies sourced from the hopr-impls
CORE-1168 Git branch; Cargo.lock pins their revision. Remove the workspace Git
patches when the companion crates are released.

Run the `state_dump` integration test with `--features testing,state-dump`. It runs
runtime enabled/disabled configurations in isolated processes. It verifies real INFO
records from an emulated node cluster with channel
inventory, a routed echo session, routing/search/SURB/probe activity, continuing
periodic emission, and the payload budget. This complements unit tests for cache
hits, failed searches, probe expiry and independent SURB counter windows. Live
production CPU and log-volume measurements remain rollout validation.
