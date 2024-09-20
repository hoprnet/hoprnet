# Introduction

The best way to learn and develop the HOPR protocol is by interacting with a HOPR node connected inside the HOPR network. A series of HOPR nodes interconnected (i.e. reachable and with channels opened against each other) is called a HOPR cluster.

Although the HOPR Association provides a [production HOPR cluster](https://status.hoprnet.org/) anyone could talk to in production[^1],
this setup is not ideal for development as doing so would incur on costs for executing basic operations. Staging networks
such as görli are too slow to provide meaningful feedback, so the ideal setup for properly interacting with the HOPR
ecosystem toolset is running a local HOPR cluster, either in your workstation or via a cloud environment like
[Gitpod](https://gitpod.io). This way contributors are not constrained by internet connection or blockchain RPC providers.

# Tooling

As of the time of writing, the best way to set up a local HOPR cluster is by following these steps[^2]. Please bear in mind that these steps
had been tested only on `Darwin` and `Linux` devices, for Windows please use our cloud setup instead.

1. **Download the latest version of the repository**: Download a local version of our [GitHub repository monorepo](https://github.com/hoprnet/hoprnet)[^3]
   and unzip it in your local folder and enter the unzipped directory.

```
wget https://github.com/hoprnet/hoprnet/archive/refs/heads/master.zip
unzip master.zip
cd hoprnet-master
```

1. **Install the dependencies of the project and build it**:

```
make -j deps && make -j build
```

2. **Run the one-line setup script**: Proceed to run the following script.

```
./scripts/setup-local-cluster.sh -i topologies/full_interconnected_cluster.sh
```

**Important**, make sure to have both `curl` and `jq` installed in your computer before running the script, as both are used.
Please be aware you also need a version of `bash` of `5.x` or superior, which in most macOS devices require an upgrade, the easiest being via `brew bash`.

Afterwards, a set off accounts with their respective HTTP REST API, HOPR Admin, and WebSocket interface will be displayed
in your screen. If this is your first time using HOPR, I suggest navigating to the `HOPR Admin` URL to get familiar with
the basic commands. Afterwards, it might also make sense to check the API v2 Swagger URL.

[^1]:
    Production is to be understood as a "live" network where [HOPR](https://coinmarketcap.com/currencies/hopr/) tokens can
    be exchanged or used as means to power its network `HoprChannels.sol` smart contract. As of time of writing, the only live
    network is the [Gnosis Chain](https://www.xdaichain.com/) (previously known as xDai Chain), with the only available publicly
    traded utility token being [`wxHOPR`](https://blockscout.com/xdai/mainnet/token/0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1/token-transfers).

[^2]:
    Tasks [#167](https://github.com/hoprnet/hopr-devrel/issues/167), [#168](https://github.com/hoprnet/hopr-devrel/issues/168),
    and [#169](https://github.com/hoprnet/hopr-devrel/issues/167) are alternatives to the current setup. Whenever these tasks are
    completed, these instructions will be updated in [#170](https://github.com/hoprnet/hopr-devrel/issues/170).

[^3]:
    By using the `master` tag, you are downloading the latest version of `hoprnet` to spin up your nodes, which might be ideal
    to your particular use case. However, due to the rapid development done on the project, you might be better off using a stable
    release, found on the [`Release` page](https://github.com/hoprnet/hoprnet/releases).
