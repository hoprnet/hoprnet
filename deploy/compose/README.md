# `docker compose` based deployment

## Requirements

- A VPS or cloud provider VM with docker and docker compose installed
- Ensure the P2P port (default 9091) is opened from your router or internet gateway to your VPS to allow network communications.

## Setup

The `docker compose` deployment is multi-faceted, allowing different combinations of tooling and extensions and different types of usage for the deployment. Please follow the guide to [set up a node](https://docs.hoprnet.org/node/node-docker-compose).

### Profiles

The `docker compose` setup is profile driven. Based on which profiles are activated, functionality is unlocked, whereas each profile must be activated explicitly to allow that functionality.

The supported profiles are:

- `hoprd`: runs a single hoprd node with configuration taken from config file
  - requires the `./hoprd_data/hoprd.cfg.yaml` to be edited with relevant information
  - takes the `./hoprd_data/hopr.id` file or generates a new id encrypted with `HOPRD_PASSWORD` from `./env-secrets`
- `admin-ui`: runs a `hopr-admin` frontend
- `metrics`: utilites exporting system, docker and node metrics
- `metrics-push`: a utility cronjob to publish metrics to an external prometheus push gateway
- `metrics-vis`: visualization tools for the metrics (containing the prometheus and grafana setup with default dashboards)
- `tracing`: Enable Jaeger tracing to forward hopr traces to Jaeger. Remind to enable the environment variable `HOPRD_USE_OPENTELEMETRY` before.

Profiles should be specified as a list of `,` separated values in the `COMPOSE_PROFILES` environment variable.

#### Examples

Inside the copied compose directory:

1. Run only the hopr node

```shell
COMPOSE_PROFILES=hoprd docker compose up -d
```

2. Run the `hopr-admin` and a hopr node

```shell
COMPOSE_PROFILES=hoprd,admin-ui docker compose up -d
```

Access the website at [http://localhost:4677](http://localhost:4677), where `HOPR_ADMIN_PORT=4677` is the configured port.
The default hoprd endpoint is available at [http://localhost:3001](http://localhost:3001), with `HOPRD_API_PORT=3001` as the configured port.

3. Run hopr node with a full internal monitoring system (Prometheus and Grafana)

```shell
COMPOSE_PROFILES=hoprd,metrics-vis docker compose up -d
```

To access Prometheus, navigate to [http://localhost:9090](http://localhost:9090), where `PROMETHEUS_PORT=9090` is the configured port.
To access Grafana, navigate to [http://localhost:3030](http://localhost:3030), where `GRAFANA_PORT=3030` is the configured port.
Grafana credentials are stored in ./grafana/config.monitoring
Navigate to the Dashboards page and open the desired dashboard

4. Run hopr node with an external monitoring system using Prometheus pushgateway

Before running this profile, make sure that you modify the variable `METRICS_PUSH_URL` to point to your prometheus pushgateway instance and that you name your hoprd node accordingly among other nodes.

```shell
COMPOSE_PROFILES=hoprd,metrics-push docker compose up -d
```

5. Run everything

```shell
COMPOSE_PROFILES=hoprd,admin-ui,metrics,metrics-vis docker compose up -d
```

The same list of `COMPOSE_PROFILES` should be supplied for the `docker compose down` command.
