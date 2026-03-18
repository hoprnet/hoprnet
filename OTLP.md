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
- They are not exposed by the Prometheus `/metrics` endpoint.
