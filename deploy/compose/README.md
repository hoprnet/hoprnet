## `docker compose` based deployment

The `docker compose` deployment is multi-faceted allowing different combinations of tooling and extensions and different types of usage for the deployment.

Copy this directory to where the setup should run and rename the directory appropriately.

Once in the directory, copy and setup the `.env` file:

```shell
cp .env{.example,}
```

Now edit the `.env` file if any override is necessary.

### Profiles

The `docker compose` setup is profile driven. Based on which profiles are activated, functionality is unlocked, whereas each profile must be activated explicitly to allow that functionality.

The supported profiles are:

- `hoprd`: runs a single hoprd node with configuration taken from config file
  - requires the `./hoprd.cfg.yaml` to be edited with relevant information
  - requires the `./hopr.id` file to be supplied inside the directory
- `admin-ui`: runs a `hopr-admin` frontend
- `metrics`: utilites exporting system, docker and node metrics
- `metrics-push`: a utility cronjob to publish metrics to an external prometheus push gateway
- `metrics-vis`: visualization tools for the metrics (containing the prometheus and grafana setup with default dashboards)

Profiles should be specified as a list of `,` separated values in the `COMPOSE_PROFILES` environment variable.

#### Examples

Inside the copied compose directory:

1. run only the hopr node

```shell
COMPOSE_PROFILES=hoprd docker compose up -d
```

2. run the `hopr-admin` and a hopr node

```shell
COMPOSE_PROFILES=hoprd,admin-ui docker compose up -d
```

3. run everything

```shell
COMPOSE_PROFILES=hoprd,admin-ui,metrics,metrics-vis docker compose up -d
```

The same list of `COMPOSE_PROFILES` should be supplied for the `docker compose down` command.

### Types of usage

#### `docker compose` based HOPR node

Uses the setup as is.

#### Externally run HOPR node

Omit the `hoprd` profile and run only components deemed necessary for the externally running node.

- in `.env` set all variables so that `prometheus` looks at the proper node
