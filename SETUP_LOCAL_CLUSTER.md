# Introduction

For both HOPR app and core protocol developers, being able to spin a fully interconnected HOPR network is critical.
Without doing so, app developers will only be able to fake interactions with a HOPR node by mocking their API[^1], and
core protocol developers will not be able to replicate critical functionality over messaging such as ticket redemption
and packet relaying.

Although the HOPR Association provides a series of [nodes](https://status.hoprnet.org/) anyone could talk in production[^2],
this setup is not ideal for development as doing so would incur on costs for executing basic operations. Staging networks
such as görli are too slow to provide meaningful feedback, so the ideal setup for properly interacting with the HOPR
ecosystem toolset is running a local HOPR network, either in your workstation or via a local environment like
[Gitpod](https://gitpod.io). This way contributors are not constrained by internet connection or blockchain RPC providers.

# Tooling

As of time of writing, the best way to setup a local HOPR network is by following these steps[^3]:

1. **Download the latest version of the repository**: Download a local version of our [GitHub repository monorepo](https://github.com/hoprnet/hoprnet)[^4]
   and unzip it in your local folder (roughly `~30 Mbs` at the time of writing). For the next tasks, we will assume you are within that folder.

```
wget https://github.com/hoprnet/hoprnet/archive/refs/heads/master.zip
unzip master.zip
cd hoprnet-master
```

2. **Install the dependencies of the project and build it**: Make sure you are using `nodejs@16`, and proceed to use `yarn`
   to install the required packages and project modules. Soon afterward, build the project to use it in the next step.

```
yarn
yarn build
```

3. **Run the one-line setup script**: Proceed to run the following script. If you are planning to run [MyneChat](http://app.myne.chat/)
   alongside, then make sure to pass the `-m` flag with your MyneChat instance URL. Please wait while this script creates
   the local blockchain network and deploys the project contracts. In average, the process can take between 2-6 minutes,
   depending on your computer capacity.

```
./scripts/setup-local-cluster.sh -m "http://app.myne.chat" -i scripts/topologies/full_interconnected_cluster.sh
```

Afterwards, a set off accounts with their respective HTTP REST API, HOPR Admin, and WebSocket interface will be displayed
in your screen. If this is your first time using HOPR, I suggest navigating to the `HOPR Admin` URL to get familiar with
the basic commands. Afterwards, it might also make sense to check the API v2 Swagger URL.

[^1]:
    The demo application [MyneChat](https://github.com/hoprnet/myne-chat) uses a
    [mock server](https://github.com/hoprnet/myne-chat/blob/cf6501b2ffa24502834f567ab575630e302e3d34/mocks/index.js#L47-L79)
    to simplify it’s development workflow. Nevertheless, to fully experience the extend of its features, it relies on a
    HOPR network, either a local or a public one.

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
    release. The latest stable release known at the time of writing is [`athens`](https://github.com/hoprnet/hoprnet/archive/refs/heads/release/athens.zip).
