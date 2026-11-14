# Decision logging

The published chain, graph and P2P implementations expose optional diagnostics
through `hopr_api::StateDump`. Calling `dump().await` returns an owned, serializable
snapshot. Each implementation owns its state schema and assessment policy; callers
must not depend on private snapshot types or the removed combined collector.

For the versions used by this workspace:

- `hopr-chain-connector` 0.33.0 provides `state_view(safe_address, funding_value)`.
  The returned view implements `StateDump`; the caller supplies its Safe address and
  current funding policy, such as the graph's ticket face value.
- `hopr-network-graph` 0.12.0 implements `StateDump` on `ChannelGraph`, including
  shared handles. A separate `state_view` accepts a bounded path-sample provider.
- `hopr-transport-p2p` 0.15.0 implements `StateDump` on `HoprNetwork`.

Their diagnostic features enable the public interface. `hopr-lib/state-dump`
forwards those features when the optional implementations are enabled, including
by `testing`. It does not install a reporter automatically. On-demand polling is
independent of the periodic reporting configuration.

## Periodic component reporting

The concrete implementations also expose `run_state_reporting` methods. The chain
implementation additionally offers `run_state_reporting_with_funding_policy` to
read the caller's current funding policy on each interval. The application runs
these futures in its runtime and owns their cancellation and tracing subscriber.
Each module owns its timer, collection deadline, record budget and error handling.

`HOPR_STATE_DUMP_INTERVAL` is integer seconds: default 60, `0` disables periodic
collection, and enabled values must be 5–86400. Invalid values return an error when
the reporting future is polled. These methods emit separate INFO records with
`event=component_state`, a component name, instance identity, sequence, reason and
`analysis_required`. Their JSON `state` field contains the reporting envelope and
the module's own snapshot. The published reporters cap each serialized record at
32 KiB and apply a five-second collection deadline. Missing observations and
collection failures remain distinguishable from an observed zero.

With a JSON tracing subscriber, extract records with:

```sh
jq 'select(.fields.event == "component_state") | .fields.state | fromjson' node.jsonl
```

Component observations are not atomic with one another. Detailed schemas can
change between releases. Diagnostic `analysis_required` is separate from the
module's operational `ComponentStatus`. Logging uses the application's existing
sink; the application chooses rotation, retention and shipping.

## Node lifecycle and transport observers

`hopr-lib` also exposes `HoprBuilderConfigured::with_state_dump` for applications
that want a reporting future managed with the node's other tasks. This is a host
integration interface: `StateDumpFactory::prepare` validates the supplied factory's
configuration before component construction, and `create` receives the actual
chain, graph, network, Safe address and registered section callbacks. The returned
`hopr_lib::state_dump::StateDump` bundles an optional transport observer and a lazy
future. The node attaches the observer before transport starts, starts the future
after account readiness, and owns its cancellation.

This task wrapper is distinct from the public `hopr_api::StateDump` polling trait.
The factory chooses how to use module diagnostics and registered section callbacks;
`hopr-lib` does not prescribe their JSON layout, section limits or interval policy.
`with_state_dump_section` registers a named callback without invoking it.

Transport producers report path resolutions, graph searches and SURB mints/returns
to an optional `TransportObserver`. The events include direction, destination,
search-phase results and path pairs. The observer must be cheap and non-blocking.
The transport constructor alone installs no observer. Applications can supply one
through a reporting factory or `HoprTransport::with_observer`.

Probe outcomes remain in producer-local logs and graph telemetry. They are not
forwarded to a central state-dump counter. Returns can cross interval boundaries,
so minted/returned counts are not delivery probabilities.

The original request for a combined node record and destination/path-pair interval
summaries is not the public `StateDump` contract. This adapter migration supplies
no combined collector or replacement transport-counter store. Any remaining
operator-facing diagnostic gaps must be addressed in production implementations,
with tests owned by the relevant module.

## Log levels and verification

Routine probe ping/pong, expiry, path resolution, selector candidates/search failures,
return-path draws, SURB mint/decode and per-pair round-trip details use TRACE. Relayer
sets used only for path diagnostics are computed only when TRACE is enabled.
DEBUG retains bounded weighted-collection summaries on cache rebuild (up to eight
path/weight pairs, total weight, candidate count and omitted count), aggregate cache
updates and actionable degradation transitions. Probe replies and expiry callbacks
retain producer-local TRACE spans with pseudonym/tag or peer context. Reply spans
are created only for matched probes. SURB flushes retain one aggregate INFO event
per active interval. Named blind-spot details are capped at eight destinations,
with an omitted count. TRACE requires a build that retains that static level.

Run the `state_dump` integration test with `--features testing,state-dump` and one
test thread. The Nix integration and integration-coverage targets enable both
required features. The test polls public diagnostic views from running node roles
and a real P2P implementation, and exercises the published periodic reporters in
isolated enabled, disabled and invalid-configuration processes. It checks INFO
emission, independent sequences, assessments, the documented per-record budget,
continued idle reporting and cancellation. Component-specific snapshot contents
and aggregation policies belong to the upstream module tests. The builder's tests
separately verify factory validation ordering and lazy section registration.
