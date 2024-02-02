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
- [Usage](#usage)
  - [Example execution](#example-execution)
  - [Using Docker Compose with extended HOPR node monitoring](#using-docker-compose-with-extended-hopr-node-monitoring)
- [Testnet accessibility](#testnet-accessibility)
- [Migrating between releases](#migrating-between-releases)
- [Develop](#develop)
  - [Nix environment setup](#nix-environment-setup)
    - [Nix flake outputs](#nix-flake-outputs)
  - [Local node with safe staking service (local network)](#local-node-with-safe-staking-service-local-network)
  - [Local node with safe staking service (dufour network)](#local-node-with-safe-staking-service-dufour-network)
- [Local cluster](#local-cluster)
- [Test](#test)
  - [Unit testing](#unit-testing)
  - [Github Actions CI](#github-actions-ci)
  - [End-to-End Testing](#end-to-end-testing)
    - [Running Tests Locally](#running-tests-locally)
      - [Testing environment](#testing-environment)
      - [Test execution](#test-execution)
- [Contact](#contact)
- [License](#license)


## About

The HOPR project produces multiple artifacts that allow running, maintaining and modiyfing the HOPR node. The most relevant components for production use cases are:

1. [hopr-lib](https://hoprnet.github.io/hoprnet/hopr_lib/index.html)
    - A fully self-contained referential implementation of the HOPR protocol over a libp2p based connection mechanism that can be incroporated into another projects as a transport layer.
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

Docker images are published in a GCP registry under this registry/repository: `europe-west3-docker.pkg.dev/hoprassociation/docker-images/${PROJECT}:${RELEASE}`
Where:
- ${PROJECT} can be either `hopli` or `hoprd`
- ${RELEASE} can be: `latest` or `stable`

Here it is an example to pull the stable release for hoprd:

```shell
$ docker pull europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:stable
```

Here is an example of command line to run the docker container. Please note that some parameters should be set customized to your node configuration

```shell
$ alias hoprd='docker run --pull always --platform linux/amd64 --restart on-failure -m 3g \
  --log-driver json-file --log-opt max-size=1000M --log-opt max-file=5 \
  -d -v $HOME/.hoprd-db:/app/hoprd-db --name hoprd \
  -p 9091:9091/tcp -p 9091:9091/udp -p 3001:3001 -e RUST_LOG="info" -e RUST_BACKTRACE="full" \
  europe-west3-docker.pkg.dev/hoprassociation/docker-images/hoprd:${RELEASE}'
  
$  hoprd \
  --network dufour \
  --identity /app/hoprd-db/.hopr-id \
  --apiHost "$(curl http://ifconfig.me)" --apiToken '${HOPRD_API_TOKEN}' \
  --password '${HOPRD_PASSWORD}' \
  --safeAddress ${HOPRD_SAFE_ADDRESS} --moduleAddress ${HOPRD_MODULE_ADDRESS}
```
Where:
- ${RELEASE} is one of the docker tags mentioned before
- ${HOPRD_PASSWORD} is the password to encrypt your database 
- ${HOPRD_API_TOKEN} is A REST API token used for user authentication
- ${HOPRD_SAFE_ADDRESS} is the ethereum address obtained during the onboarding process
- ${HOPRD_MODULE_ADDRESS} is the ethereum address obtained during the onboarding process.

### Install via [Nix package manager][1]

WARNING: This setup should only be used for development or advanced usage without any further support.

Clone and initialize the [`hoprnet`](https://github.com/hoprnet/hoprnet) repository:

```shell
$ git clone https://github.com/hoprnet/hoprnet
$ cd hoprnet
```

Build and install the `hoprd` binary, e.g. on a UNIX platform:

```shell
$ nix build
$ sudo cp result/bin/* /usr/local/bin/
```

## Usage

`hoprd` provides various command-line switches to configure its behaviour. For reference these are documented here as well:

```shell
$ hoprd --help
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
      --announce
          Announce the node on chain with a public address [env: HOPRD_ANNOUNCE=]
      --api
          Expose the API on localhost:3001 [env: HOPRD_API=]
      --apiHost <HOST>
          Set host IP to which the API server will bind [env: HOPRD_API_HOST=]
      --apiPort <PORT>
          Set port to which the API server will bind [env: HOPRD_API_PORT=]
      --apiToken <TOKEN>
          A REST API token and for user authentication [env: HOPRD_API_TOKEN=]
      --password <PASSWORD>
          A password to encrypt your keys [env: HOPRD_PASSWORD=]
      --defaultStrategy <DEFAULT_STRATEGY>
          Default channel strategy to use after node starts up [env: HOPRD_DEFAULT_STRATEGY=] [possible values: promiscuous, aggregating, auto_redeeming, auto_funding, multi, passive]
      --maxAutoChannels <MAX_AUTO_CHANNELS>
          Maximum number of channel a strategy can open. If not specified, square root of number of available peers is used. [env: HOPRD_MAX_AUTO_CHANNELS=]
      --disableTicketAutoRedeem
          Disables automatic redeeming of winning tickets. [env: HOPRD_DISABLE_AUTO_REDEEEM_TICKETS=]
      --disableUnrealizedBalanceCheck
          Disables checking of unrealized balance before validating unacknowledged tickets. [env: HOPRD_DISABLE_UNREALIZED_BALANCE_CHECK=]
      --provider <PROVIDER>
          A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER=]
      --init
          initialize a database if it doesn't already exist [env: HOPRD_INIT=]
      --forceInit
          initialize a database, even if it already exists [env: HOPRD_FORCE_INIT=]
      --inbox-capacity <INBOX_CAPACITY>
          Set maximum capacity of the HOPRd inbox [env: HOPRD_INBOX_CAPACITY=]
      --testAnnounceLocalAddresses
          For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES=]
      --heartbeatInterval <MILLISECONDS>
          Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL=]
      --heartbeatThreshold <MILLISECONDS>
          Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD=]
      --heartbeatVariance <MILLISECONDS>
          Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE=]
      --networkQualityThreshold <THRESHOLD>
          Minimum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD=]
      --configurationFilePath <CONFIG_FILE_PATH>
          Path to a file containing the entire HOPRd configuration [env: HOPRD_CONFIGURATION_FILE_PATH=]
      --safeTransactionServiceProvider <HOPRD_SAFE_TX_SERVICE_PROVIDER>
          Base URL for safe transaction service [env: HOPRD_SAFE_TRANSACTION_SERVICE_PROVIDER=]
      --safeAddress <HOPRD_SAFE_ADDR>
          Address of Safe that safeguards tokens [env: HOPRD_SAFE_ADDRESS=]
      --moduleAddress <HOPRD_MODULE_ADDR>
          Address of the node mangement module [env: HOPRD_MODULE_ADDRESS=]
      --protocolConfig <HOPRD_PROTOCOL_CONFIG_PATH>
          Path to the protocol-config.json file [env: HOPRD_PROTOCOL_CONFIG_PATH=]
      --dryRun
          DEPRECATED [env: HOPRD_DRY_RUN=]
      --healthCheck
          DEPRECATED
      --healthCheckHost <HEALTH_CHECK_HOST>
          DEPRECATED
      --healthCheckPort <HEALTH_CHECK_PORT>
          DEPRECATED
  -h, --help
          Print help
  -V, --version
          Print version
```

### Example execution
Running the node without any command-line argument might not work depending on the installation method used. Some command line arguments are required.

A basic reasonable setup is that uses a custom identity and enabels a REST API of the `hoprd` could look like:

```sh
hoprd --identity /app/hoprd-db/.hopr-identity --password switzerland --init --announce --host "0.0.0.0:9091" --apiToken <MY_TOKEN> --network doufur
```

Here is a short breakdown of each argument.

```sh
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
  # an network is defined as a chain plus a number of deployed smart contract addresses to use on that chain                       
  --network doufur                        
```

Special care needs to given to the `network` argument, which defines the specific network `hoprd` node should join. Only nodes within the same network can communicate using the HOPR protocol.

### Using Docker Compose with extended HOPR node monitoring

An optional `docker compose` setup can be used to run the above containerized `hoprd` along with extension to observe the node's metrics using Prometheus + Grafana dashboard:

```shell
docker compose --file scripts/compose/docker-compose.yml up -d
```

Copy the  `scripts/compose/default.env` to `scripts/compose/.env` and change the variables as desired.

The composite setup will publish multiple additional services alongside the `hoprd`:
- Admin UI at `localhost:3000`
- Grafana with `hoprd` dashboards at `localhost:3030` (default user: `admin` and pass `hopr`)

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

Install `nix`` from the official website at [https://nix.dev/install-nix.html](https://nix.dev/install-nix.html).

Create a nix configuration file at `~/.config/nix/nix.conf` with the following content:

```nix
experimental-features = nix-command flakes
```

Install the `nix-direnv` package to introduce the `direnv`:

```shell
$ nix-env -i nix-direnv
```

Append the following line to the shell rc file (depending on the shell used it can be `~\.zshrc`, `~\.bashrc`, `~\.cshrc`, etc.). Modify the `<shell>` variable inside the below command with the currently used (`zsh`, `bash`, `csh`, etc.):

```shell
$ eval "$(direnv hook <shell>)"
```

From within the [`hoprnet`](https://github.com/hoprnet/hoprnet) repository's directory, execute the following command.

```bash
$ direnv allow .
```

#### Nix flake outputs

We provide a couple of packages, apps and shells to make building and
development easier, to get the full list execute:. You may get the full list like so:

```shell
$ nix flake show
```

### Local node with safe staking service (local network)

Running one node in test mode, with safe and module attached (in an `anvil-localhost` network)

```shell
# clean up, e.g.
# make kill-anvil
# make clean

# build deps and HOPRd code
make -j deps && make -j build

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

```shell
# build deps and HOPRd code
make -j deps && make -j build

# ensure a private key with enough xDAI is set as PRIVATE_KEY
# Please use the deployer private key as PRIVATE_KEY
# in `ethereum/contract/.env` which should be manually filled out
# from the `ethereum/contract/example.env`
source ./ethereum/contracts/.env

export HOPR_NETWORK="dufour"
export PRIVATE_KEY=${DEPLOYER_PRIVATE_KEY}
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

```shell
make test
```

For more information please refer to [act][2]'s documentation.

### End-to-End Testing

When using the `nix` environment, the test environment preparation and activation is automatic.

Tests are using the `pytest` infrastructure. 

#### Running Tests Locally

##### Testing environment
If not using `nix`, setup the `pytest` environment:

```shell
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r tests/requirements.txt
```

To deactivate the activated testing environment if no longer needed:

```shell
deactivate
```

##### Test execution

With the environment activated, execute the tests locally:

```shell
make smoke-test-full
```

## Contact

- [Twitter](https://twitter.com/hoprnet)
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



