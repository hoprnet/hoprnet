# `docker compose` based deployment

## Requirements

- A VPS or cloud provider VM with docker and docker compose installed
- Ensure the P2P port (default 9091) is opened from your router or internet gateway to your VPS to allow network communications.

## Setup

### Setup a node for production

The `docker compose` deployment is multi-faceted, allowing different combinations of tooling and extensions and different types of usage for the deployment. Please follow the guide to [set up a node](https://docs.hoprnet.org/node/node-docker-compose).

### Setup a node for testing

This guide will walk you through the steps required to set up a node on the Rotsee network for testing purposes.

#### 1. Create an identity and make it eligible for the Rotsee network

Before running a node on the Rotsee network, ensure you complete these steps: create an identity file, set up a safe, configure module instances, and link the identity to those instances.

For detailed instructions, follow [this guide](https://github.com/hoprnet/hopli/blob/main/README.md#create-read-identity-and-make-it-eligible-for-rotsee-network).

#### 2. Clone hoprnet repository

Start by cloning `hoprnet` repository:

```
git clone https://github.com/hoprnet/hoprnet.git
```

#### 3. Set up environment variables

Navigate to the `/deploy/compose` folder and rename `.env.sample` to `.env`.

Adjust the following environment variables as needed:

- `HOPRD_API_PORT`: Adjust REST API port, default is `3001`.
- `HOPRD_P2P_PORT`: Adjust the p2p communication port, default is `9091`.
- `HOPRD_IMAGE`: Change the image tag from `stable` to `latest`. Example: `europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:latest`.

Configure the Metrics url:

Update the `METRICS_PUSH_URL` variable with the following data:

- `<NODE_NAME_FOR_GRAFANA>`: Your node's name for easy identification, example: `node-chaos`.
- `<NODE_PEER_ID>`: The PeerID, To get the PeerID from your identity file, follow [7th step](https://github.com/hoprnet/hopli/blob/main/README.md#create-read-identity-and-make-it-eligible-for-rotsee-network).
- `<MY_NAMESPACE>`: Adjust the namespace to better categorize. Suggestion: `team`.
- `<NODE_NATIVE_ADDRESS>`: Native address, to get the native address from your identity file, follow [7th step](https://github.com/hoprnet/hopli/blob/main/README.md#create-read-identity-and-make-it-eligible-for-rotsee-network).

Example: `https://prometheus-pushgateway.staging.hoprnet.link/metrics/job/node-chaos/hoprd_peer_id/12D3KooWBX5ENqYkKTevwiQXpTFYZTWs3S11bQiueX7QQqapTsaR/namespace/team/hoprd_network/rotsee/hoprd_address/0xb6b099780bdf1c1041783178b88be8c9449e75ab`

#### 4. Set up secrets environment variables

In the `compose` folder, rename `.env-secrets.sample` to `.env-secrets`.

Adjust the following secrets:

- `HOPRD_PASSWORD`: Replace `<YOUR HOPRD IDENTITY PASSWORD>` with the identity password you created in the [1st step](#1-create-identity-and-make-it-eligible-for-rotsee-network).

- `HOPRD_API_TOKEN`: Replace `<YOUR HOPRD API TOKEN>` with your own security token to connect via HOPR Admin UI or REST API.

#### 5. Edit docker compose yml file

To test GnosisVPN, you need to have an open session with a node. Follow these steps to make the necessary changes:

- `services.hoprd.ports`: Add an additional UDP port under the ports variable to open a session. For example: `11111:11111/udp`

#### 6. Configure the Configuration File

Inside the `compose` folder, navigate to the `hoprd_data` folder and edit the `hoprd.cfg.yaml` file. Update the following fields:

- `host.address`: Set the public IP address of the machine where the node is running.
- `host.port`: Use the P2P port which you configured in [3rd step](#3-set-up-environment-variables).
- `chain.network`: Change `dufour` to `rotsee`.
- `chain.provider`: Enter the RPC endpoint URL. If using a local RPC endpoint, include the http:// prefix with the IP address or localhost.
- `safe_module.safe_address`: Enter the safe address created in a [1st step](#1-create-identity-and-make-it-eligible-for-rotsee-network).
- `safe_module.module_address`: Enter the module address created in a [1st step](#1-create-identity-and-make-it-eligible-for-rotsee-network).

#### 7. Upload the identity file

After creating the identity file in [1st step](#1-create-identity-and-make-it-eligible-for-rotsee-network), rename the file to `hopr.id` and copy it to the `/compose/hoprd_data/` folder.

#### 8. Launch the node

The Docker Compose setup uses profiles. To start a node, ensure you are inside the compose folder so the profile functionality can be recognized. In this case we will start Docker compose using the `hoprd` profile (HOPR node) and the `metrics-push` profile (to push metrics to prometheus).

Execute command: `COMPOSE_PROFILES=hoprd,metrics-push docker compose up -d`

For more details on different profiles, refer to [this section](#profiles).

## Docker Compose usage

### Profiles

The `docker compose` setup is profile driven. Based on which profiles are activated, functionality is unlocked, whereas each profile must be activated explicitly to allow that functionality.

The supported profiles are:

- `hoprd`: runs a single hoprd node with configuration taken from config file
  - requires the `./hoprd/conf/hoprd.cfg.yaml` to be edited with relevant information
  - takes the `./hoprd/conf/hopr.id` file or generates a new id encrypted with `HOPRD_PASSWORD` from `./env-secrets`
- `admin-ui`: runs a `hopr-admin` frontend
- `metrics`: utilites exporting system, docker and node metrics
- `metrics-push`: a utility cronjob to publish metrics to an external prometheus push gateway
- `metrics-vis`: visualization tools for the metrics (containing the prometheus and grafana setup with default dashboards)
- `tracing`: Enable Jaeger tracing to forward hopr traces to Jaeger. Remind to enable the environment variable `HOPRD_USE_OPENTELEMETRY` before.

Profiles should be specified as a list of `,` separated values in the `COMPOSE_PROFILES` environment variable.

### Examples

#### Starting services

Commands needs to be executed inside the `compose` directory:

1. Run only the hopr node

```
COMPOSE_PROFILES=hoprd docker compose up -d
```

2. Run the `hopr-admin` and a hopr node

```
COMPOSE_PROFILES=hoprd,admin-ui docker compose up -d
```

Access the website at [http://localhost:4677](http://localhost:4677), where `HOPR_ADMIN_PORT=4677` is the configured port.
The default hoprd API endpoint is available at [http://localhost:3001](http://localhost:3001), with `HOPRD_API_PORT=3001` as the configured port.

3. Run hopr node with a full internal monitoring system (Prometheus and Grafana)

```
COMPOSE_PROFILES=hoprd,metrics-vis docker compose up -d
```

To access Prometheus, navigate to [http://localhost:9090](http://localhost:9090), where `PROMETHEUS_PORT=9090` is the configured port.
To access Grafana, navigate to [http://localhost:3030](http://localhost:3030), where `GRAFANA_PORT=3030` is the configured port.
Grafana credentials are stored in ./grafana/config.monitoring
Navigate to the Dashboards page and open the desired dashboard

4. Run hopr node with an external monitoring system using Prometheus pushgateway

Before running this profile, make sure that you modify the variable `METRICS_PUSH_URL` to point to your prometheus pushgateway instance and that you name your hoprd node accordingly among other nodes. Modify the variable `METRICS_PUSH_KEY` to set the user and password available for the Prometheus Pushgateway. Get it from Bitwarden Secret 'Prometheus Pushgateway Hoprd Node'.

```
COMPOSE_PROFILES=hoprd,metrics-push docker compose up -d
```

5. Run everything

```
COMPOSE_PROFILES=hoprd,admin-ui,metrics,metrics-vis docker compose up -d
```

#### Stopping services

Important: You must use exactly the same `COMPOSE_PROFILES` values for the `docker compose down` command as you used for `up`. Using different profiles may leave containers running or fail to clean up resources properly.

For example, if you started with:

```
COMPOSE_PROFILES=hoprd,metrics-push docker compose up -d
```

You must stop with:

```
COMPOSE_PROFILES=hoprd,metrics-push docker compose down
```
