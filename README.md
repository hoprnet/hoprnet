<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>

  <!-- Title Placeholder -->
  <h3 align="center">HOPR</h3>
  <p align="center">
    <code>A project by the HOPR Association</code>
  </p>
  <p align="center">
    HOPR is a privacy-preserving messaging protocol which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod" alt="Gitpod">
  </p>
</p>

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Getting Started](#getting-started)
- [Install](#install)
  - [Install via Docker](#install-via-docker)
  - [Install via Nix package manager](#install-via-nix-package-manager)
- [Using](#using)
  - [Using Docker](#using-docker)
  - [Using Docker Compose with extended HOPR node monitoring](#using-docker-compose-with-extended-hopr-node-monitoring)
- [Testnet accessibility](#testnet-accessibility)
- [Migrating between releases](#migrating-between-releases)
- [Develop](#develop)
- [Local cluster](#local-cluster)
- [Test](#test)
  - [Unit testing](#unit-testing)
    - [Test-driven development](#test-driven-development)
  - [Github Actions CI](#github-actions-ci)
  - [End-to-End Testing](#end-to-end-testing)
    - [Running Tests Locally](#running-tests-locally)
      - [Testing environment](#testing-environment)
      - [Test execution](#test-execution)
- [Deploy](#deploy)
  - [Using Google Cloud Platform](#using-google-cloud-platform)
  - [Using Google Cloud Platform and a Default Topology](#using-google-cloud-platform-and-a-default-topology)
- [Tooling](#tooling)
- [Contact](#contact)
- [License](#license)

## Getting Started

A good place to start is the
[Getting Started guide on YouTube][7] which walks through the following
instructions using GitPod.

## Install

The following instructions show how the latest community release may be
installed. The instructions should be adapted if you want to use the latest
development release or any other older release.

The preferred way of installation should be via Docker.

### Install via Docker

All our docker images can be found in [our Google Cloud Container Registry][4].
Each image is prefixed with `gcr.io/hoprassociation/$PROJECT:$RELEASE`.
The `rotsee` tag represents the `master` branch, while the `bratislava` tag
represents the most recent `release/*` branch.

You can pull the Docker image like so:

```sh
docker pull gcr.io/hoprassociation/hoprd:bratislava
```

For ease of use you can set up a shell alias to run the latest release as a docker container:

```sh
alias hoprd='docker run --pull always -m 2g -ti -v ${HOPRD_DATA_DIR:-$HOME/.hoprd-db}:/app/db -p 9091:9091/tcp -p 9091:9091/udp -p 3001:3001 gcr.io/hoprassociation/hoprd:bratislava'
```

**IMPORTANT:** Using the above command will map the database folder used by hoprd to a local folder called `.hoprd-db` in your home directory. You can customize the location of that folder further by executing the following command:

```sh
HOPRD_DATA_DIR=${HOME}/.hoprd-better-db-folder eval hoprd
```

Also all ports are mapped to your localhost, assuming you stick to the default port numbers.

### Install via [Nix package manager][1]

NOTE: This setup should only be used for development or if you know what you
are doing and don't need further support. Otherwise you should use the `npm`
or `docker` setup.

You will need to clone and initialize the `hoprnet` repo first:

```sh
git clone https://github.com/hoprnet/hoprnet
cd hoprnet
make init
```

If you have [direnv][2] set up properly your `nix-shell` will be
configured automatically upon entering the `hoprnet` directory and enabling it
via `direnv allow`. Otherwise you must enter the `nix-shell` manually:

```sh
nix develop
```

Now you may follow the instructions in [Develop](#develop).

Alternatively you may use a development Docker container which uses the same Nix
setup.

```
make run-docker-dev
```

## Using

The `hoprd` provides various command-line switches to configure its behaviour. For reference these are documented here as well:

```sh
$ hoprd --help
Options:
      --network <NETWORK>
          Network id which the node shall run on [env: HOPRD_NETWORK=] [possible values: anvil-localhost, rotsee, debug-staging, anvil-localhost2, monte_rosa]
      --identity <identity>
          The path to the identity file [env: HOPRD_IDENTITY=] [default: <IDENTITY_DIR>]
      --data <data>
          manually specify the data directory to use [env: HOPRD_DATA=] [default: <DATA_DIR>]
      --host <HOST>
          Host to listen on for P2P connections [env: HOPRD_HOST=] [default: 0.0.0.0:9091]
      --announce
          Run as a Public Relay Node (PRN) [env: HOPRD_ANNOUNCE=]
      --api
          Expose the API on localhost:3001 [env: HOPRD_API=]
      --apiHost <HOST>
          Set host IP to which the API server will bind [env: HOPRD_API_HOST=] [default: localhost]
      --apiPort <PORT>
          Set port to which the API server will bind [env: HOPRD_API_PORT=] [default: 3001]
      --apiToken <TOKEN>
          A REST API token and for user authentication [env: HOPRD_API_TOKEN=]
      --healthCheck
          Run a health check end point on localhost:8080 [env: HOPRD_HEALTH_CHECK=]
      --healthCheckHost <HOST>
          Updates the host for the healthcheck server [env: HOPRD_HEALTH_CHECK_HOST=] [default: localhost]
      --healthCheckPort <PORT>
          Updates the port for the healthcheck server [env: HOPRD_HEALTH_CHECK_PORT=] [default: 8080]
      --password <PASSWORD>
          A password to encrypt your keys [env: HOPRD_PASSWORD=]
      --defaultStrategy <DEFAULT_STRATEGY>
          Default channel strategy to use after node starts up [env: HOPRD_DEFAULT_STRATEGY=] [default: passive] [possible values: promiscuous, passive, random]
      --maxAutoChannels <MAX_AUTO_CHANNELS>
          Maximum number of channel a strategy can open. If not specified, square root of number of available peers is used. [env: HOPRD_MAX_AUTO_CHANNELS=]
      --disableTicketAutoRedeem
          Disables automatic redeemeing of winning tickets. [env: HOPRD_DISABLE_AUTO_REDEEEM_TICKETS]
      --disableUnrealizedBalanceCheck
          Disables checking of unrealized balance before validating unacknowledged tickets. [env: HOPRD_DISABLE_UNREALIZED_BALANCE_CHECK]
      --provider <PROVIDER>
          A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER=]
      --dryRun
          List all the options used to run the HOPR node, but quit instead of starting [env: HOPRD_DRY_RUN=]
      --init
          initialize a database if it doesn't already exist [env: HOPRD_INIT=]
      --forceInit
          initialize a database, even if it already exists [env: HOPRD_FORCE_INIT=]
      --allowLocalNodeConnections
          Allow connections to other nodes running on localhost [env: HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS=]
      --allowPrivateNodeConnections
          Allow connections to other nodes running on private addresses [env: HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS=]
      --maxParallelConnections <CONNECTIONS>
          Set maximum parallel connections [env: HOPRD_MAX_PARALLEL_CONNECTIONS=] [default: 50000]
      --testAnnounceLocalAddresses
          For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES=]
      --noRelay
          disable NAT relay functionality entirely [env: HOPRD_NO_RELAY=]
      --heartbeatInterval <MILLISECONDS>
          Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL=] [default: 60000]
      --heartbeatThreshold <MILLISECONDS>
          Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD=] [default: 60000]
      --heartbeatVariance <MILLISECONDS>
          Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE=] [default: 2000]
      --onChainConfirmations <CONFIRMATIONS>
          Number of confirmations required for on-chain transactions [env: HOPRD_ON_CHAIN_CONFIRMATIONS=] [default: 8]
      --networkQualityThreshold <THRESHOLD>
          Miniumum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD=] [default: 0.5]
      --safeAddress <HOPRD_SAFE_ADDRESS>
          The Safe instance for a node where its HOPR tokens are held [env: HOPRD_SAFE_ADDRESS=]
      --moduleAddress <HOPRD_MODULE_ADDRESS>
          The node management module instance that manages node permission to assets held in safe [env: HOPRD_MODULE_ADDRESS=]
  -h, --help
          Print help
  -V, --version
          Print version

All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.
```

As you might have noticed running the node without any command-line argument might not work depending on the installation method used. Here are examples to run a node with some safe configurations set.

### Using Docker

The following command assumes you've setup an alias like described in [Install via Docker](#install-via-docker).

```sh
hoprd --identity /app/hoprd-db/.hopr-identity --password switzerland --init --announce --host "0.0.0.0:9091" --apiToken <MY_TOKEN> --network monte_rosa
```

Here is a short breakdown of each argument.

```sh
hoprd
  --identity /app/hoprd-db/.hopr-identity     # store your node identity information in the persisted database folder
  --password switzerland   	                  # set the encryption password for your identity
  --init 				                              # initialize the database and identity if not present
  --announce 				                          # announce the node to other nodes in the network and act as relay if publicly reachable
  --host "0.0.0.0:9091"   	                  # set IP and port of the P2P API to the container's external IP so it can be reached on your host
  --apiToken <MY_TOKEN>                       # specify password for accessing REST API(REQUIRED)
  --network monte_rosa                        # an network is defined as a chain plus a number of deployed smart contract addresses to use on that chain
                                              # each release has a default network id set, but the user can override this value
                                              # nodes from different networks are **not able** to communicate
```

### Using Docker Compose with extended HOPR node monitoring

There is an optional Docker Compose setup that can be used to run the above Docker image with HOPRd and also
have an extended monitoring of the HOPR node's activity (using Prometheus + Grafana dashboard).

To startup a HOPRd node with monitoring, you can use the following command:

```shell
docker compose --file scripts/compose/docker-compose.yml up -d
```

The configuration of the HOPRd node can be changed in the `scripts/compose/default.env` file.

Once the configuration starts up, the HOPRd Admin UI is accessible as usual via `localhost:3000`. The Grafana instance is
accessible via `localhost:3030` and is provisioned with a dashboard that contains useful metrics and information
about the HOPR network as perceived from your node plus some additional runtime information.

The default username for Grafana is `admin` with password `hopr`.

## Testnet accessibility

Currently, to be able to participate in a public testnet or public staging environment, you need to satisfy certain criteria to be eligible to join. See [Network Registry](NETWORK_REGISTRY.md) for details.

These criteria however, are not required when you develop using your local nodes or a locally running cluster (see [Develop section below](#develop)).

## Migrating between releases

At the moment we DO NOT HAVE backward compatibility between releases.
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

HOPR contains modules written in Rust, therefore a Rust toolchain is needed to successfully build the artifacts.
To install Rust toolchain (at least version 1.60) please follow instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) first.

```sh
# build deps and HOPRd code
make -j deps && make -j build

# starting network
make run-anvil

# update protocol-config
scripts/update-protocol-config.sh -n anvil-localhost

# running normal node alice (separate terminal)
DEBUG="hopr*" yarn run:hoprd:alice

# running normal node bob (separate terminal)
DEBUG="hopr*" yarn run:hoprd:bob

# fund all your nodes to get started
make fund-local-all

# start local HOPR admin in a container (and put into background)
make run-hopr-admin &
```

### Local node with safe staking service (local network)

Running one node in test mode, with safe and module attached (in anvil-localhost network)

```sh
# clean up, e.g.
# make kill-anvil
# make clean

# build deps and HOPRd code
make -j deps && make -j build

# starting network
make run-anvil

# update protocol-config
scripts/update-protocol-config.sh -n anvil-localhost

# create identity files
make create-local-identity

# create a safe and a node management module instance,
# and passing the created safe and module as argument to
# run a test node local (separate terminal)
# It also register the created pairs in network registry, and
# approve tokens for channels to move token.
# fund safe with 2k token and 1 native token
make run-local-with-safe
# or to restart a node and use the same id, safe and module
# run:
# make run-local id_path=$(find `pwd` -name ".identity-local*.id" | sort -r | head -n 1)

# fund all your nodes to get started
make fund-local-all id_dir=`pwd`

# start local HOPR admin in a container (and put into background)
make run-hopr-admin &
```

### Local node with safe staking service (rotsee network)

Running one node in test mode, with safe and module attached (in rotsee network)

```sh
# build deps and HOPRd code
make -j deps && make -j build

# ensure a private key with enough xDAI is set as PRIVATE_KEY
# Please use the deployer private key as PRIVATE_KEY
# in `packages/ethereum/contract/.env`
source ./packages/ethereum/contracts/.env

# create identity files
make create-local-identity

# create a safe and a node management module instance,
# and passing the created safe and module as argument to
# run a test node local (separate terminal)
# It also register the created pairs in network registry, and
# approve tokens for channels to move token.
# fund safe with 2k wxHOPR and 1 xdai
make run-local-with-safe-rotsee network=rotsee
# or to restart a node and use the same id, safe and module
# run:
# make run-local network=rotsee id_path=$(find `pwd` -name ".identity-local*.id" | sort -r | head -n 1)

# fund all your nodes to get started
make fund-local-rotsee id_dir=`pwd`


# start local HOPR admin in a container (and put into background)
make run-hopr-admin &
```

## Local cluster

The best way to test with multiple HOPR nodes is by using a local cluster of interconnected nodes.
See [how to start your local HOPR cluster](SETUP_LOCAL_CLUSTER.md).

## Test

### Unit testing

We use [mocha][9] for our tests. You can run our test suite across all
packages using the following command:

```sh
make test
```

To run tests of a single package (e.g. hoprd) execute:

```sh
make test package=hoprd
```

To run tests of a single test suite (e.g. Identity) within a
package (e.g. hoprd) execute:

For instance, to run only the `Identity` test suite in `hoprd`, you need to
run the following:

```sh
yarn --cwd packages/hoprd test --grep "Identity"
```

In a similar fashion, our contracts can be tested in isolation. For now, you
need to pass the file to be tested, as [hardhat does not support --grep][12]

```sh
yarn test:contracts test/HoprChannels.spec.ts
```

In case a package you need to test is not included in our `package.json`,
please feel free to update it as needed.

#### Test-driven development

To make sure we add the least amount of untested code to our codebase,
whenever possible all code should come accompanied by a test. To do so,
locate the `.spec` or equivalent test file for your code. If it does not
exist, create it within the same file your code will live in.

Afterwards, ensure you create a breaking test for your feature. For example,
the [following commit][10] added a test to a non-existing feature. The
immediate [commit][11] provided the actual feature for that given test. Repeat
this process for all the code you add to our codebase.

_(The code was pushed as an example, but ideally, you only push code that has
working tests on your machine, as to avoid overusing our CI pipeline with
known broken tests.)_

### Github Actions CI

We run a fair amount of automation using Github Actions. To ease development
of these workflows one can use [act][8] to run workflows locally in a
Docker environment.

E.g. running the build workflow:

```sh
act -j build
```

For more information please refer to [act][8]'s documentation.

### End-to-End Testing

#### Running Tests Locally

##### Testing environment

Tests are using the `pytest` infrastructure that can be set up inside a virtualenv using as:

```sh
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -r tests/requirements.txt
```

To deactivate the activated testing environment if no longer needed:

```sh
deactivate
```

##### Test execution

With the environment activated, execute the tests locally:

```sh
python3 -m pytest tests/
```

## Deploy

The deployment nodes and networks are mostly orchestrated through the script
files in `scripts/` which are executed by the Github Actions CI workflows.
Therefore, all common and minimal networks do not require manual steps to be
deployed.

### Using Google Cloud Platform

However, sometimes it is useful to deploy additional nodes or specific versions
of `hoprd`. To accomplish that its possible to create a cluster on GCP using the
following scripts:

```sh
./scripts/setup-gcloud-cluster.sh my-custom-cluster-without-name
```

Read the full help information of the script in case of questions:

```sh
./scripts/setup-gcloud-cluster.sh --help
```

The script requires a few environment variables to be set, but will inform the
user if one is missing. It will create a cluster of 6 nodes. By default these
nodes will use the latest Docker image of `hoprd` and run on the `Goerli`
network. Different versions and different target networks can be configured
through the parameters and environment variables.

To launch nodes using the `xDai` network one would execute (with the
placeholders replaced accordingly):

```sh
HOPRD_API_TOKEN="<ADMIN_AUTH_HTTP_TOKEN>" \
HOPRD_PASSWORD="<IDENTITY_FILE_PASSWORD>" \
  ./scripts/setup-gcloud-cluster.sh network "" my-custom-cluster-without-name
```

A previously started cluster can be destroyed, which includes all running nodes,
by using the same script but setting the cleanup switch:

```sh
HOPRD_PERFORM_CLEANUP=true \
  ./scripts/setup-gcloud-cluster.sh network "" my-custom-cluster-without-name
```

The default Docker image in `scripts/setup-gcloud-cluster.sh` deploys GCloud public nodes. If you wish to deploy GCloud nodes
that are behind NAT, you need to specify a NAT-variant of the `hoprd` image (note the `-nat` suffix in the image name):

```sh
HOPRD_PERFORM_CLEANUP=true \
  ./scripts/setup-gcloud-cluster.sh network "" my-nat-cluster gcr.io/hoprassociation/hoprd-nat
```

### Using Google Cloud Platform and a Default Topology

The creation of a `hoprd` cluster on GCP can be enhanced by providing a topology
script to the creation script:

```sh
./scripts/setup-gcloud-cluster.sh \
  my-custom-cluster-without-name \
  gcr.io/hoprassociation/hoprd:bratislava \
  `pwd`/scripts/topologies/full_interconnected_cluster.sh
```

After the normal cluster creation the topology script will then open channels
between all nodes so they are fully interconnected. Custom topology scripts can
be easily added and used in the same manner. Refer to the referenced scripts as
a guideline on how to get started.

## Tooling

As some tools are only partially supported, please tag the respective team member
whenever you need an issue about a particular tool.

| Maintainer | Technology |
| :--------- | :--------: |
| @tolbrino  |    Nix     |

## Contact

- [Twitter](https://twitter.com/hoprnet)
- [Telegram](https://t.me/hoprnet)
- [Medium](https://medium.com/hoprnet)
- [Reddit](https://www.reddit.com/r/HOPR/)
- [Email](mailto:contact@hoprnet.org)
- [Discord](https://discord.gg/5FWSfq7)
- [Youtube](https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA)

## License

[GPL v3](LICENSE) © HOPR Association

[1]: https://nixos.org/learn.html
[2]: https://search.nixos.org/packages?channel=20.09&show=direnv&from=0&size=50&sort=relevance&query=direnv
[4]: https://console.cloud.google.com/gcr/images/hoprassociation/GLOBAL
[6]: https://www.npmjs.com/package/@hoprnet/hoprd
[7]: https://www.youtube.com/watch?v=d0Eb6haIUu4
[8]: https://github.com/nektos/act
[9]: https://mochajs.org/
[10]: https://github.com/hoprnet/hoprnet/pull/1974/commits/331d6e99d1199250a302211be7b8dd9a22fa6e23#diff-83e70acfe04a8f13821ff96a1115f02a4b683a6370568ba9beea16da6d0c2cffR33-R49
[11]: https://github.com/hoprnet/hoprnet/pull/1974/commits/53663517309d0f8918c5066fd98503afe8d8dd76#diff-9bf7c02325c8f5b6330a15a745a3ad736ee139a78c28a15d594756c406378884R91-R96
[12]: https://github.com/nomiclabs/hardhat/issues/1116
