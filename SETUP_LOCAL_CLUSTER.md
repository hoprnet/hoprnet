# Introduction

The best way to learn and develop the HOPR protocol is by interacting with a HOPR node connected inside the HOPR network. A series of HOPR nodes interconnected (i.e. reachable and with channels opened against each other) is called a HOPR cluster.
Without a HOPR cluster, app developers can only fake interactions with a HOPR node by mocking their API[^1], and
core protocol developers can't replicate critical functionality over messaging such as ticket redemption
and packet relaying.

Although the HOPR Association provides a [production HOPR cluster](https://status.hoprnet.org/) anyone could talk to in production[^2],
this setup is not ideal for development as doing so would incur on costs for executing basic operations. Staging networks
such as görli are too slow to provide meaningful feedback, so the ideal setup for properly interacting with the HOPR
ecosystem toolset is running a local HOPR cluster, either in your workstation or via a cloud environment like
[Gitpod](https://gitpod.io). This way contributors are not constrained by internet connection or blockchain RPC providers.

# Tooling

As of the time of writing, the best way to set up a local HOPR cluster is by following these steps[^3]. Please bear in mind that these steps
had been tested only on `Darwin` and `Linux` devices, for Windows please use our cloud setup instead.

1. **Download the latest version of the repository**: Download a local version of our [GitHub repository monorepo](https://github.com/hoprnet/hoprnet)[^4]
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

1. **Run the one-line setup script**: Proceed to run the following script. If you are planning to run [MyneChat](http://app.myne.chat/)
   alongside, then make sure to pass the `-m` flag with your MyneChat instance URL. Please wait while this script creates
   the local blockchain network and deploys the project contracts. In average, the process can take between 2-6 minutes,
   depending on your computer capacity. **Important**, make sure to have both `curl` and `jq` installed in your computer
   before running the script, as both are used. Please be aware you also need a version of `bash` of `5.x` or superior,
   which in most macOS devices require an upgrade, the easiest being via `brew bash`.

```
./scripts/setup-local-cluster.sh -m "http://app.myne.chat" -i topologies/full_interconnected_cluster.sh
```

Afterwards, a set off accounts with their respective HTTP REST API, HOPR Admin, and WebSocket interface will be displayed
in your screen. If this is your first time using HOPR, I suggest navigating to the `HOPR Admin` URL to get familiar with
the basic commands. Afterwards, it might also make sense to check the API v2 Swagger URL.

[^1]:
    The demo application [MyneChat](https://github.com/hoprnet/myne-chat) uses a
    [mock server](https://github.com/hoprnet/myne-chat/blob/cf6501b2ffa24502834f567ab575630e302e3d34/mocks/index.js#L47-L79)
    to simplify it’s development workflow. Nevertheless, to fully experience the extend of its features, it relies on a
    HOPR cluster, either a local or a public one.

[^2]:
    Production is to be understood as a "live" network where [HOPR](https://coinmarketcap.com/currencies/hopr/) tokens can
    be exchanged or used as means to power its network `HoprChannels.sol` smart contract. As of time of writing, the only live
    network is the [Gnosis Chain](https://www.xdaichain.com/) (previously known as xDai Chain), with the only available publicly
    traded utility token being [`wxHOPR`](https://blockscout.com/xdai/mainnet/token/0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1/token-transfers).

[^3]:
    Tasks [#167](https://github.com/hoprnet/hopr-devrel/issues/167), [#168](https://github.com/hoprnet/hopr-devrel/issues/168),
    and [#169](https://github.com/hoprnet/hopr-devrel/issues/167) are alternatives to the current setup. Whenever these tasks are
    completed, these instructions will be updated in [#170](https://github.com/hoprnet/hopr-devrel/issues/170).

[^4]:
    By using the `master` tag, you are downloading the latest version of `hoprnet` to spin up your nodes, which might be ideal
    to your particular use case. However, due to the rapid development done on the project, you might be better off using a stable
    release. The latest stable release known at the time of writing is [`saint-louis`](https://github.com/hoprnet/hoprnet/archive/refs/heads/release/saint-louis.zip).

