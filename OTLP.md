# OTLP Configuration

This document describes how to configure OpenTelemetry (OTLP) export for `hoprd`.

## Environment Variables

| Variable                       | Required | Description                                                                                                                                   | Example                   |
| ------------------------------ | -------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------- |
| `HOPRD_USE_OPENTELEMETRY`      | No       | Enables/disables OTLP export. Accepted true values: `1`, `true`, `yes`, `on`.                                                                 | `true`                    |
| `HOPRD_OTEL_SIGNALS`           | No       | Comma-separated signals to export. Supported: `traces`, `logs`, `metrics`. Defaults to `traces`.                                              | `traces,logs,metrics`     |
| `HOPRD_OTLP_ENDPOINT`          | No       | Base OTLP endpoint. Transport is inferred from scheme (`grpc://`, `http://`, `https://`). Internally mapped to `OTEL_EXPORTER_OTLP_ENDPOINT`. | `http://localhost:4318`   |
| `HOPRD_METRIC_EXPORT_INTERVAL` | No       | Metric export cadence config: `default,prefix=interval[,prefix=interval...]`. Intervals support ms integer or `ms/s/m` suffixes.              | `15000,hopr_session=1000` |

## Metric Interval Syntax

`HOPRD_METRIC_EXPORT_INTERVAL` supports:

- Default interval for all metrics (first value without `=`).
- Prefix overrides for selected metric names.

Examples:

- `15000`
  - all OTLP metrics exported every 15s
- `15000,hopr_session=1000`
  - default OTLP metrics every 15s
  - metrics whose names start with `hopr_session` every 1s
- `10s,hopr_session=1s,hopr_http_api=5s`
  - default every 10s
  - session metrics every 1s
  - HTTP API metrics every 5s

## Endpoint Notes

`HOPRD_OTLP_ENDPOINT` is the user-facing variable. At startup, `hoprd` sets `OTEL_EXPORTER_OTLP_ENDPOINT` internally from `HOPRD_OTLP_ENDPOINT`. If both are set and differ, `HOPRD_OTLP_ENDPOINT` takes precedence.

## Example Configurations

### Local HTTP OTLP Ingestor (metrics only)

```bash
HOPRD_USE_OPENTELEMETRY=true
HOPRD_OTEL_SIGNALS=metrics
HOPRD_OTLP_ENDPOINT=http://localhost:4318
HOPRD_METRIC_EXPORT_INTERVAL=10000,hopr_session=1000
```

### Jaeger gRPC (all signals)

```bash
HOPRD_USE_OPENTELEMETRY=true
HOPRD_OTEL_SIGNALS=traces,logs,metrics
HOPRD_OTLP_ENDPOINT=grpc://jaeger:4317
HOPRD_METRIC_EXPORT_INTERVAL=15000,hopr_session=1000
```

## Session Metrics

- Session telemetry metrics are exported via OTLP (`hopr_session_*`).
- When a `hopr_session` prefix override is configured in `HOPRD_METRIC_EXPORT_INTERVAL` — which the
  examples below recommend — they are not exposed by the Prometheus `/metrics` endpoint either. Without such an
  override no prefix provider exists for them, and they fall through to the default provider, which does have the
  Prometheus reader.

## PIX Exit Metrics

The PIX Exit surface is split across both exporters, by consumer rather than by subsystem:

- **`hopr_pix_*`** — bounded node-level aggregates: live Sessions and SSA cycles by phase, predeposit exposure,
  egress and gate pressure, share coverage, cycle outcomes, live-cycle capacity and admission refusals, plus the
  per-reason closure (`hopr_pix_closures_total`) and fill-backoff (`hopr_pix_fill_backoff_total`) counters. None
  is labelled by a Session, cycle, peer or address, so these are the ones to dashboard and alert on. They go to
  the default provider and therefore **do** appear on `/metrics`.
- **`hopr_session_pix_*`** — per-Session detail: gate mode, recovery progress, fill rate. OTLP only once a
  `hopr_session` interval override is configured, like the rest of `hopr_session_*` (see the Session Metrics
  section), because each is labelled by a Session.

Which of the two prefixes a metric belongs under is decided by whether it is bounded by construction, not by
which subsystem raised it.

One PIX-adjacent pair deliberately sits outside this section: `hopr_surb_reserve_holds_total` and
`hopr_surb_reserve_hold_seconds`, the Exit data held by the SURB keep-alive reserve. The hold runs before the PIX
gate, so it also covers rate-controlled Sessions without PIX, and it is not labelled by a Session — so it belongs
on the `hopr_surb_*` node-level family and **does** appear on `/metrics`. It is the counterpart to
`hopr_pix_gate_blocks_total`, not another spelling of it. Both are labelled by `pix` = `true|false` (whether the
Session runs a PIX gate), a closed set, so a PIX Session's hold is separable from the non-PIX case.

It is also not the same signal as `hopr_pix_fill_backoff_total{reason="surb_reserve"}`, which counts the keep-alives
the Exit withheld from its own PIX fill schedule. The new pair counts _data packets held for the Entry_, on any
rate-controlled Session. Both rise during an SURB shortage; summing them as one pressure signal double-counts.

The routing is by name prefix, so the spelling is what decides the exporter. The full metric contract —
instrument type, unit, the exact transition that updates each one, its complete label value set, and worked
PromQL for the questions above — is the module documentation of
`transport/session/src/telemetry/pix.rs` and `transport/session/src/telemetry/surb.rs`.

Note that per-Session series are subject to the OpenTelemetry SDK's 2000-per-instrument cardinality limit (see
issue #8305); the `hopr_pix_*` aggregates are not, by construction.
