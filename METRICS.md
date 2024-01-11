# Metrics

This file is documenting and tracking all the metrics which can be collected
by a Prometheus server. The metrics can be scraped by Prometheus from
the `api/v2/node/metrics` API endpoint.

## Example Prometheus configuration

An example scrape config for Prometheus to collect HOPRd metrics:

```yaml
scrape_configs:
  - job_name: 'hoprd'
    scrape_interval: 5s
    static_configs:
      - targets: ['localhost:3001']
    metrics_path: /api/v2/node/metrics
    basic_auth:
      username: ^MYtoken4testing^
      password: ''
```
