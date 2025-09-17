<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img align="middle" width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>

  <!-- Title Placeholder -->
  <h3 align="center">HOPR</h3>
  <p align="center">
    <code>A project by the HOPR Association</code>
  </p>
  <p align="center">
    HOPR is a privacy-preserving messaging protocol which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.
  </p>
</p>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [About](#about)
- [Install](#install)
  - [Install via Docker](#install-via-docker)
  - [Install via Nix package manager](#install-via-nix-package-manager)
  - [Install via linux package manager](#install-via-linux-package-manager)
- [Usage](#usage)
  - [Environment variables](#environment-variables)
  - [Example execution](#example-execution)
  - [Using Docker Compose with extended HOPR node monitoring](#using-docker-compose-with-extended-hopr-node-monitoring)
  - [REST API](#rest-api)
- [Testnet accessibility](#testnet-accessibility)
- [Migrating between releases](#migrating-between-releases)
- [Develop](#develop)
  - [Nix environment setup](#nix-environment-setup)
    - [Nix flake outputs](#nix-flake-outputs)
    - [Code Formatting](#code-formatting)
    - [Code Linting](#code-linting)
    - [Generate the Python SDK](#generate-the-python-sdk)
  - [Building a Docker image](#building-a-docker-image)
  - [Local node with safe staking service (local network)](#local-node-with-safe-staking-service-local-network)
  - [Local node with safe staking service (dufour network)](#local-node-with-safe-staking-service-dufour-network)
- [Local cluster](#local-cluster)
- [Test](#test)
  - [Unit testing](#unit-testing)
  - [Github Actions CI](#github-actions-ci)
  - [End-to-End Testing](#end-to-end-testing)
    - [Running Tests Locally](#running-tests-locally)
      - [Test execution](#test-execution)
- [Using Fast Sync](#using-fast-sync)
  - [Prerequisites](#prerequisites)
  - [Database Files](#database-files)
  - [Configuration Steps](#configuration-steps)
  - [Post-sync Behavior](#post-sync-behavior)
- [Profiling \& Instrumentation](#profiling--instrumentation)
  - [`tokio` executor instrumentation](#tokio-executor-instrumentation)
  - [OpenTelemetry tracing](#opentelemetry-tracing)
  - [HOPR packet capture](#hopr-packet-capture)
- [Contact](#contact)
- [License](#license)

## About

The HOPR project produces multiple artifacts that allow running, maintaining and modifying the HOPR node.
The most relevant components for production use cases are:

1. [hopr-lib](https://hoprnet.github.io/hoprnet/hopr_lib/index.html)
   - A fully self-contained referential implementation of the HOPR protocol over a libp2p based connection mechanism that can be incorporated into other projects as a transport layer.
2. [hoprd](https://hoprnet.github.io/hoprnet/hoprd/index.html)
   - Daemon application providing a higher level interface for creating a HOPR protocol compliant node that can use a dedicated REST API.
3. [hoprd-api-schema](https://hoprnet.github.io/hoprnet/hoprd_api_schema/index.html)
   - Utility to generate the OpenAPI spec for the `hoprd` served REST API.
4. [hoprd-cfg](https://hoprnet.github.io/hoprnet/hoprd_cfg/index.html)
   - Utility for configuration management of the `hoprd`
5. [hopli](https://hoprnet.github.io/hoprnet/hopli/index.html)
   - Utility designed to simplify and unify the management of on-chain and identity related tasks.

Unless stated otherwise, the following sections only apply to `hoprd`.

## Install

For production purposes always run the latest stable release.

Multiple options for installation exist, the preferred choice for any production system should be to use the container image (e.g. using `docker`).

All releases and associated changelogs are located in the [official releases](https://github.com/hoprnet/hoprnet/releases) section of the [`hoprnet`](https://github.com/hoprnet/hoprnet) repository.

### Install via Docker

The following instructions show how any `$RELEASE` may be installed, to select the release, override the `$RELEASE` variable, e.g.:

- `export RELEASE=latest` to track the latest changes on the repository's `master` branch
- `export RELEASE=kaunas` to track the latest changes on the repository's `release/kaunas` branch (3.0.X)
- `export RELEASE=<version>` to get a specific `<version>`

Container image has the format
`europe-west3-docker.pkg.dev/hoprassociation/docker-images/$PROJECT:$RELEASE`.
where:

- `$PROJECT` can be either `hopli` or `hoprd`

Pull the container image with `docker`:

```bash
docker pull europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:kaunas
```

It is recommended to setup an alias `hoprd` for the docker command invocation.

### Install via [Nix package manager][1]

WARNING: This setup should only be used for development or advanced usage without any further support.

Clone and initialize the [`hoprnet`](https://github.com/hoprnet/hoprnet) repository:

```bash
git clone https://github.com/hoprnet/hoprnet
cd hoprnet
```

Build and install the `hoprd` binary, e.g. on a UNIX platform:

```bash
nix build
sudo cp result/bin/* /usr/local/bin/
```

To build and access man pages for `hoprd` and `hopli`:

```bash
# Build man page for hoprd
nix build .#hoprd-man
man ./result/share/man/man1/hoprd.1.gz

# Build man page for hopli
nix build .#hopli-man
man ./result/share/man/man1/hopli.1.gz

# Or install them system-wide
sudo cp -r result/share/man/man1/* /usr/local/share/man/man1/
```

### Install via linux package manager

Linux packages are available at every github release, download the latest package from https://github.com/hoprnet/hoprnet/releases/latest
To install on specific distribution, see [detailed information](./deploy/nfpm/README.md)

## Usage

`hoprd` provides various command-line switches to configure its behaviour. For reference these are documented here as well:

```bash
$ hoprd --help
HOPR node executable.

Usage: hoprd [OPTIONS]

Options:
      --network <NETWORK>
          ID of the network the node will attempt to connect to [env: HOPRD_NETWORK=]
      --identity <IDENTITY>
          The path to the identity file [env: HOPRD_IDENTITY=]
      --data <DATA>
          Specifies the directory to hold all the data [env: HOPRD_DATA=]
      --host <HOST>
          Host to listen on for P2P connections [env: HOPRD_HOST=]
      --announce...
          Announce the node on chain with a public address [env: HOPRD_ANNOUNCE=]
      --api...
          Expose the API on localhost:3001 [env: HOPRD_API=]
      --apiHost <HOST>
          Set host IP to which the API server will bind [env: HOPRD_API_HOST=]
      --apiPort <PORT>
          Set port to which the API server will bind [env: HOPRD_API_PORT=]
      --defaultSessionListenHost <DEFAULT_SESSION_LISTEN_HOST>
          Default Session listening host for Session IP forwarding [env: HOPRD_DEFAULT_SESSION_LISTEN_HOST=]
      --apiToken <TOKEN>
          A REST API token and for user authentication [env: HOPRD_API_TOKEN=]
      --password <PASSWORD>
          A password to encrypt your keys [env: HOPRD_PASSWORD=]
      --noKeepLogs...
          Disables keeping RPC logs in the logs database after they were processed. [env: HOPRD_INDEXER_DISABLE_KEEP_LOGS=]
      --noFastSync...
          Disables using fast sync at node start. [env: HOPRD_INDEXER_DISABLE_FAST_SYNC=]
      --enableLogsSnapshot...
          Enables downloading logs snapshot at node start. If this is set to true, the node will attempt to download logs snapshot from the configured `logsSnapshotUrl`. [env: HOPRD_ENABLE_LOGS_SNAPSHOT=]
      --logsSnapshotUrl <LOGS_SNAPSHOT_URL>
          URL to download logs snapshot from. If none is provided or configured in the configuration file, the node will not attempt to download any logs snapshot. [env: HOPRD_LOGS_SNAPSHOT_URL=]
      --maxBlockRange <MAX_BLOCK_RANGE>
          Maximum number of blocks that can be fetched in a batch request from the RPC provider. [env: HOPRD_MAX_BLOCK_RANGE=]
      --maxRequestsPerSec <MAX_RPC_REQUESTS_PER_SEC>
          Maximum number of RPC requests that can be performed per second. [env: HOPRD_MAX_RPC_REQUESTS_PER_SEC=]
      --provider <PROVIDER>
          A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER=]
      --init...
          initialize a database if it doesn't already exist [env: HOPRD_INIT=]
      --forceInit...
          initialize a database, even if it already exists [env: HOPRD_FORCE_INIT=]
      --probeRecheckThreshold <SECONDS>
          Timeframe in seconds after which it is reasonable to recheck the nearest neighbor [env: HOPRD_PROBE_RECHECK_THRESHOLD=]
      --networkQualityThreshold <THRESHOLD>
          Minimum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD=]
      --configurationFilePath <CONFIG_FILE_PATH>
          Path to a file containing the entire HOPRd configuration [env: HOPRD_CONFIGURATION_FILE_PATH=]
      --safeTransactionServiceProvider <HOPRD_SAFE_TX_SERVICE_PROVIDER>
          Base URL for safe transaction service [env: HOPRD_SAFE_TRANSACTION_SERVICE_PROVIDER=]
      --safeAddress <HOPRD_SAFE_ADDR>
          Address of Safe that safeguards tokens [env: HOPRD_SAFE_ADDRESS=]
      --moduleAddress <HOPRD_MODULE_ADDR>
          Address of the node management module [env: HOPRD_MODULE_ADDRESS=]
      --protocolConfig <HOPRD_PROTOCOL_CONFIG_PATH>
          Path to the protocol-config.json file [env: HOPRD_PROTOCOL_CONFIG_PATH=]
  -h, --help
          Print help
  -V, --version
          Print version
```

### Environment variables

On top of the default configuration options generated for the command line, the following environment variables can be used in order to tweak the node functionality:

- `ENV_WORKER_THREADS` - the number of environment worker threads for the tokio executor
- `HOPRD_LOG_FORMAT` - override for the default stdout log formatter (follows tracing formatting options)
- `HOPRD_USE_OPENTELEMETRY` - enable the OpenTelemetry output for this node
- `OTEL_SERVICE_NAME` - the name of this node for the OpenTelemetry service
- `HOPR_INTERNAL_LIBP2P_MAX_CONCURRENTLY_DIALED_PEER_COUNT` - the maximum number of concurrently dialed peers in libp2p
- `HOPR_INTERNAL_LIBP2P_MAX_NEGOTIATING_INBOUND_STREAM_COUNT` - the maximum number of negotiating inbound streams
- `HOPR_INTERNAL_LIBP2P_SWARM_IDLE_TIMEOUT` - timeout for all idle libp2p swarm connections in seconds
- `HOPR_INTERNAL_DB_PEERS_PERSISTENCE_AFTER_RESTART_IN_SECONDS` - cutoff duration from now to not retain the peers with older records in the peers database (e.g. after a restart)
- `HOPR_INTERNAL_REST_API_MAX_CONCURRENT_WEBSOCKET_COUNT` - the maximum number of concurrent websocket opened through the REST API
- `HOPR_INTERNAL_MIXER_CAPACITY` - capacity of the mixer buffer
- `HOPR_INTERNAL_MIXER_MINIMUM_DELAY_IN_MS` - the minimum mixer delay in milliseconds
- `HOPR_INTERNAL_MIXER_DELAY_RANGE_IN_MS` - the maximum range of the mixer delay from the minimum value in milliseconds
- `HOPR_BALANCER_PID_P_GAIN` - proportional (P) gain for the PID controller in outgoing SURB balancer (default: `0.6`)
- `HOPR_BALANCER_PID_I_GAIN` - integral (I) gain for the PID controller in outgoing SURB balancer (default: `0.7`)
- `HOPR_BALANCER_PID_D_GAIN` - derivative (D) gain for the PID controller in outgoing SURB balancer (default: `0.2`)
- `HOPR_SURB_RB_SIZE` - number of incoming SURBs the ring buffer can hold (default: 10 000)
- `HOPR_TEST_DISABLE_CHECKS` - the node is being run in test mode with some safety checks disabled (currently: minimum winning probability check)
- `HOPR_CAPTURE_PACKETS` - allow capturing customized HOPR packet format to a PCAP file or to a `udpdump` host. Note that `hoprd` must be built with the `capture` feature.
- `HOPR_TRANSPORT_MAX_CONCURRENT_PACKETS` - maximum number of concurrently processed incoming packets from all peers (default: 10)
- `HOPR_TRANSPORT_STREAM_OPEN_TIMEOUT_MS` - maximum time (in milliseconds) to wait until a stream connection is established to a peer (default: `2000 ms`)
- `HOPR_PACKET_PLANNER_CONCURRENCY` - maximum number of concurrently planned outgoing packets (default: `10`)
- `HOPR_SESSION_FRAME_SIZE` - The maximum chunk of data that can be written to the Session's input buffer (default: 1500)
- `HOPR_SESSION_FRAME_TIMEOUT_MS` - The maximum time (in milliseconds) for an incomplete frame to stay in the Session's output buffer (default: 800 ms)
- `HOPR_PROTOCOL_SURB_RB_SIZE` - size of the SURB ring buffer (default: 10 000)
- `HOPR_PROTOCOL_SURB_RB_DISTRESS` - threshold since number of SURBs in the ring buffer triggers a distress packet signal (default: 1000)
- `HOPRD_SESSION_PORT_RANGE` - allows restricting the port range (syntax: `start:end` inclusive) of Session listener automatic port selection (when port 0 is specified)
- `HOPRD_SESSION_ENTRY_UDP_RX_PARALLELISM` - sets the number of UDP listening sockets for UDP sessions on Entry node (defaults to number of CPU cores)
- `HOPRD_SESSION_EXIT_UDP_RX_PARALLELISM` - sets the number of UDP listening sockets for UDP sessions on Exit node (defaults to number of CPU cores)
- `HOPRD_NAT` - indicates whether the host is behind a NAT and sets transport-specific settings accordingly (default: `false`)

### Example execution

Running the node without any command-line argument might not work depending on the installation method used. Some command line arguments are required.

Some basic reasonable setup uses a custom identity and enables the REST API of the `hoprd`:

```bash
hoprd --identity /app/hoprd-db/.hopr-identity --password switzerland --init --announce --host "0.0.0.0:9091" --apiToken <MY_TOKEN> --network doufur
```

Here is a short breakdown of each argument.

```bash
hoprd
  # store your node identity information in the persisted database folder
  --identity /app/hoprd-db/.hopr-identity
  # set the encryption password for your identity
  --password switzerland
  # initialize the database and identity if not present
  --init
  # announce the node to other nodes in the network and act as relay if publicly reachable
  --announce
  # set IP and port of the P2P API to the container's external IP so it can be reached on your host
  --host "0.0.0.0:9091"
  # specify password for accessing REST API
  --apiToken <MY_TOKEN>
  # a network is defined as a chain plus a number of deployed smart contract addresses to use on that chain
  --network doufur
```

Special care needs to be given to the `network` argument, which defines the specific network `hoprd` node should join. Only nodes within the same network can communicate using the HOPR protocol.

### Using Docker Compose with extended HOPR node monitoring

Please follow the documentation for [`docker compose` based deployment](./deploy/compose/README.md).

### REST API

`hoprd` running a REST API exposes an endpoint at `/api-docs/openapi.json` with full OpenApi specification of the used REST API, including the current version of the API.

## Testnet accessibility

To participate in a public network the node must be eligible. See [Network Registry](https://github.com/hoprnet/hoprnet/blob/master/NETWORK_REGISTRY.md) for details.

Node eligibility is not required in a local development cluster (see [Develop section below](#develop)).

## Migrating between releases

There is **NO** backward compatibility between releases.

We attempt to provide instructions on how to migrate your tokens between releases.

1. Set your automatic channel strategy to `passive`.
2. Redeem all unredeemed tickets.
3. Close all open payment channels.
4. Once all payment channels have closed, withdraw your funds to an external
   wallet.
5. Run `info` and take note of the **network name**.
6. Once funds are confirmed to exist in a different wallet, backup `.hopr-identity` folder.
7. Launch new `HOPRd` instance using latest release, observe the account address.
8. Only transfer funds to new `HOPRd` instance if `HOPRd` operates on the **same network** as last release, you can compare the two networks using `info`.

## Develop

Either setup `nix` and `flake` to use the nix environment, or [install Rust toolchain](https://www.rust-lang.org/tools/install) from the `rust-toolchain.toml`, as well as `foundry-rs` binaries (`forge`, `anvil`).

### Nix environment setup

Install `nix` from the official website at [https://nix.dev/install-nix.html](https://nix.dev/install-nix.html).

Create a nix configuration file at `~/.config/nix/nix.conf` with the following content:

```nix
experimental-features = nix-command flakes
```

Install the `nix-direnv` package to introduce the `direnv`:

```bash
nix-env -i nix-direnv
```

Append the following line to the shell rc file (depending on the shell used it can be `~\.zshrc`, `~\.bashrc`, `~\.cshrc`, etc.). Modify the `<shell>` variable inside the below command with the currently used (`zsh`, `bash`, `csh`, etc.):

```bash
eval "$(direnv hook <shell>)"
```

From within the [`hoprnet`](https://github.com/hoprnet/hoprnet) repository's directory, execute the following command.

```bash
direnv allow .
```

#### Nix flake outputs

We provide a couple of packages, apps and shells to make building and
development easier. You may get the full list like so:

```bash
nix flake show
```

#### Code Formatting

All nix, rust, solidity and python code can be automatically formatted:

```bash
nix fmt
```

These formatters are also automatically run as a Git pre-commit check.

#### Code Linting

All linters can be executed via a Nix flake helper app:

```bash
nix run .#lint
```

This will in particular run `clippy` for the entire Rust codebase.

#### Generate the Python SDK

A Python SDK is not distributed but can be generated to connect to the HOPRd API using the [generate-python-sdk.sh](/scripts/generate-python-sdk.sh) script.

Prerequisites:

- swagger-codegen3
- build the repository to get the `hoprd-api-schema` generated

The generated SDK will be available in the `/tmp/hoprd-sdk-python/` directory. Modify the script to generate SDKs for different programming languages supported by swagger-codegen3.

For usage examples of the generated SDK, refer to the generated README.md file in the SDK directory.

### Building a Docker image

Docker images can be built using the respective nix flake outputs.
The available images can be listed with:

```bash
just list-docker-images
```

The following command builds the `hoprd` image for the host platform:

```bash
nix build .#hoprd-docker
```

If needed images for other platforms can be built by specifying the target
platform. For example, to build the `hoprd` image for the `x86_64-linux`
platform, while being on a Darwin host system, use the following command:

```bash
nix build .#packages.x86_64-linux.hoprd-docker
```

NOTE: Building for different platforms requires nix distributed builds to be
set up properly.
See [Nix documentation](https://nix.dev/manual/nix/2.28/advanced-topics/distributed-builds.html)
for more information.

### Local node with safe staking service (local network)

Running one node in test mode, with safe and module attached (in an `anvil-localhost` network)

```bash
nix run .#lint
# clean up, e.g.
# make kill-anvil
# make clean

# build HOPRd code
cargo build

# starting network
make run-anvil args="-p"

# update protocol-config
scripts/update-protocol-config.sh -n anvil-localhost

# create identity files
make create-local-identity id_count=1

# create a safe and a node management module instance,
# and passing the created safe and module as argument to
# run a test node local (separate terminal)
# It also register the created pairs in network registry, and
# approve tokens for channels to move token.
# fund safe with 2k token and 1 native token
make run-local-with-safe id_file_path=/tmp
# or to restart a node and use the same id, safe and module
# run:
# make run-local id_path=$(find `pwd` -name ".identity-local*.id" | sort -r | head -n 1)

# fund all your nodes to get started
make fund-local-all id_dir=`pwd`

# start local HOPR admin in a container (and put into background)
make run-hopr-admin &
```

### Local node with safe staking service (dufour network)

Running one node in test mode, with safe and module attached (in dufour network)

```bash
# HOPRd code
cargo build

# Fill out the `ethereum/contract/.env` from the `ethereum/contracts/example.env`
#
# ensure a private key with enough xDAI is set as PRIVATE_KEY
# This PRIVATE_KEY is the "admin_key" (i.e. owner of the created safe and node management module)
#
# Please use the deployer private key as DEPLOYER_PRIVATE_KEY
# The Ethereum address to the DEPLOYER_PRIVATE_KEY should be a "manager" of the network registry.
# Role can be checked in the explorer:

# echo "https://gnosisscan.io/address/$(jq '.networks.dufour.addresses.network_registry' ./ethereum/contracts/contracts-addresses.json)\#readContract"

source ./ethereum/contracts/.env

export HOPR_NETWORK="dufour"
export IDENTITY_PASSWORD="SOmeranDOmPassHere-DefiniteLyChangeThis!"

# create identity files
bash scripts/generate-identity.sh

# start local HOPR admin in a container (and put into background)
make run-hopr-admin &
```

## Local cluster

The best way to test with multiple HOPR nodes is by using a [local cluster of interconnected nodes](https://github.com/hoprnet/hoprnet/blob/master/SETUP_LOCAL_CLUSTER.md).

## Test

### Unit testing

Tests both the Rust and Solidity code.

```bash
make test
```

### Github Actions CI

We run a fair amount of automation using Github Actions. To ease development
of these workflows one can use [act][2] to run workflows locally in a
Docker environment.

E.g. running the build workflow:

```bash
act -j build
```

For more information please refer to [act][2]'s documentation.

### End-to-End Testing

When using the `nix` environment, the test environment preparation and activation is automatic.

Tests are using the `pytest` infrastructure.

#### Running Tests Locally

##### Test execution

With the environment activated, execute the tests locally:

```bash
just run-smoke-test integration
```

## Using Fast Sync

Fast sync is a feature that allows the node to sync the blockchain state faster
than the default sync mode by using a pre-built logs database.

### Prerequisites

To generate the logs database, you need:

- A fully synced node
- Node configured to keep logs in the database (enabled by default)
  - Set `hopr -> chain -> keep_logs` in the configuration file

### Database Files

The following files in the node's database folder are required:

- `hopr_logs.db` - Main logs database
- `hopr_logs.db-shm` - Shared memory file (auxiliary)
- `hopr_logs.db-wal` - Write-Ahead Log file (auxiliary)

### Configuration Steps

1. Place the pre-built logs database files in the node's database folder
2. Enable fast sync mode (enabled by default):

- Set `hopr -> chain -> fast_sync` to `true` in the configuration file

3. Remove any existing index data:

   ```bash
   rm hopr_index.db*
   ```

### Post-sync Behavior

- If index data exists but is incomplete, the node will resume fast sync at the
  last processed log
- If index data exists and is complete, the node will skip fast sync and start in normal sync mode
- After fast sync completes, the node automatically switches to normal sync mode

## Profiling & Instrumentation

Multiple layers of profiling and instrumentation can be used to debug the `hoprd`:

### `tokio` executor instrumentation

Requires a special build:

1. Set `RUSTFLAGS="--cfg tokio_unstable"` before building
2. Enable the `profiling` feature on the `hoprd` package: `cargo build --feature profiling`

Once an instrumented tokio is built into hoprd, the application can be instrumented by `tokio_console` as described in the [official crate documentation](https://docs.rs/tokio-console/latest/tokio_console/#instrumenting-the-application).

### OpenTelemetry tracing

`hoprd` is adapted to stream OpenTelemetry to a compatible endpoint. This behavior is turned off by default. To enable it, these environment variables have to be specified:

- `HOPRD_USE_OPENTELEMETRY` - `true` to enable the OpenTelemetry streaming, `false` to disable it
- `OTEL_SERVICE_NAME` - the identifier used to assign traces from this instance to (e.g. `my_hoprd_instance`)
- `OTEL_EXPORTER_OTLP_ENDPOINT` - URL of an endpoint accepting the OpenTelemetry format (e.g. http://jaeger:4317/)

### Profiling Criterion benchmarks via `flamegraph`

#### Prerequisites

- `perf` installed on the host system
- flamegraph (install via e.g. `cargo install flamegraph`)

#### Profiling the benchmarking binaries

1. Perform a build of your chosen benchmark with `--no-rosegment` linker flag:

   ```
   RUSTFLAGS="-Clink-arg=-fuse-ld=lld -Clink-arg=-Wl,--no-rosegment" cargo bench --no-run -p hopr-crypto-packet
   ```

   Use `mold` instead of `lld` if needed.

2. Find the built benchmarking binary and check if it contains debug symbols:

   ```
   readelf -S target/release/deps/packet_benches-ce70d68371e6d19a | grep debug
   ```

   The output of the above command should contain AT LEAST: `.debug_line`, `.debug_info` and `.debug_loc`

3. Run `flamegraph` on the benchmarking binary of a selected benchmark with a fixed profile time (e.g.: 30 seconds):
   ```
   flamegraph -- ./target/release/deps/packet_benches-ce70d68371e6d19a --bench --exact packet_sending_no_precomputation/0_hop_0_surbs --profile-time 30
   ```
4. The `flamegraph.svg` will be generated in the project root directory and can be opened in a browser.

### HOPR packet capture

Using the environment variable `HOPR_CAPTURE_PACKETS` allows capturing customized HOPR packet format to a PCAP file or to a `udpdump` host.
However, for that to work the `hoprd` binary has to be built with the feature `capture`.
For ease of use we provide different nix flake outputs that build the `hoprd`
with the `capture` feature enabled:

- `nix build .#hoprd-x86_64-linux-profile`
- `nix build .#hoprd-aarch64-linux-profile`
- `nix build .#hoprd-x86_64-darwin-profile`
- `nix build .#hoprd-aarch64-darwin-profile`

## Contact

- [X](https://x.com/hoprnet)
- [Telegram](https://t.me/hoprnet)
- [Medium](https://medium.com/hoprnet)
- [Reddit](https://www.reddit.com/r/HOPR/)
- [Email](mailto:contact@hoprnet.org)
- [Discord](https://discord.gg/5FWSfq7)
- [Youtube](https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA)

## License

[GPL v3](https://github.com/hoprnet/hoprnet/blob/master/LICENSE) © HOPR Association

[1]: https://nixos.org/learn.html
[2]: https://github.com/nektos/act
