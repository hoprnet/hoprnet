@hoprnet/hopr-cover-traffic-daemon / [Exports](modules.md)

# Cover traffic node

Cover traffic (CT) node is a daemon built with HOPR's core-protocol (`hopr-core`), which allows it to join the HOPR network in the same way as other HOPR nodes. What distinguishs a CT node from other default HOPR nodes is that:

- CT node maintains a persisted state locally, which records the topology of the network from its own perspective and the latest status of CT delivery to facilitate the management of CT channels and path-selection for cover traffic.
- CT node adopts `CoverTrafficStrategy`. At each strategy tick, CT strategy decides which channels to be opened/closed based on the current publicly-available network state. Those channels are referred as cover traffic channels (CT channels). At each tick, destination of all the open CT channels are served as the first hop for CT node to send looped traffic.
- CT node is an application of HOPR core. It does not expose any `sendMessage()` methods and is not designed to be used by end users.

### Persisted state

At run time, network topology and CT traffic status are saved into a data file `./ct.json`. Note that it must live in same timeline as the hoprdb, as it relies on the indexer being in the same state.

### Open/close CT channels

At each tick, CT strategy decides which channel to open and which to close:

#### Close channels

Among all the existing open CT channels, a CT channel will be closed on a strategy tick, when

- the destination node has a network quality lower than the `CT_NETWORK_QUALITY_THRESHOLD` threshold (default, `0.15`).
- it does not have enough stake.
- its failed traffic rate reaches the `MESSAGE_FAIL_THRESHOLD` threshold (default, `10`).
- it stalls at `WAIT_FOR_COMMITMENT` state for too long.

#### Open channels

While filtering channels to close, CT strategy also selects nodes to which it opens CT channels based on the probability weighted by importance, if the total amount of CT channels does not reach the `CHANNELS_PER_COVER_TRAFFIC_NODE` limit (default: 10). Nodes must:

- not be the CT node itself.
- not be the destination of CT channels from the lastest update.

For the definition of "importance", please check [§6 HOPR yellow paper](https://github.com/hoprnet/hoprnet/blob/master/docs/yellowpaper/yellowpaper.pdf)

### Metrics

Some metrics can be extracted from the logs, please check [metrics.md](./metrics.md)

## Installation

```
yarn install
yarn build
```

## Run

```sh
$ hopr-cover-traffic-daemon --help
Options:
  --help        Show help  [boolean]
  --version     Show version number  [boolean]
  --provider    A provider url for the network this node shall operate on  [default: "https://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/"]
  --privateKey  A private key to be used for the node  [string]
  --environment Choose the environment where this node runs in. "hardhat-localhost", "hardhat-localhost2", "master-goerli", "master-xdai"
```

Example

```sh
DEBUG="hopr*" node ./lib/index.js --privateKey 0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332
```
