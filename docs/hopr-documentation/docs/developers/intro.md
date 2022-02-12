---
id: intro
title: Developing HOPR apps
---

The HOPR network is only as useful as the data that is being transfered through it. The more data is transported
through HOPR nodes, the more private it is. This is due to the architecture of the HOPR network to have constant
traffic, making it harder for malicious actors to substract meaningful information from it.

HOPR apps are a key component in that equation. Developers can build applications on top of the HOPR network to
provide users with a private channel for exchanging information. At the same time, HOPR apps create traffic within
the network, increasing the amount of noise mixers use to protect the entire ecosystem.

There are two ways of building applications with HOPR protocol:

- Building dApps on top of the REST API
- Buildling protocol applications directly with the TypeScript source code
  ![HOPR protocol stack](/img/developer/architecture.jpg)

### Use cases

Here are some of the use cases we believe the HOPR network is a great tool for:

- Browsing information securely from web sites without being tracked by ISP or third-party providers.
- Sending blockchain transactions with leaking metadata to miners or other relayers.
- Creating p2p applications that require private communication (e.g. gaming, online chats).
- Proxying traffic via a SOCKS-like interface that forwards traffic via the HOPR network.

### Building HOPR apps

The easiest way to build HOPR apps is by launching your own local HOPR cluster, connecting to them via a REST/WebSocket
client, and building on top of the REST API. Use our Walkthrough to get familiar with this entire process, and read our
OpenAPI documentation to learn how to interact with HOPR nodes once they are up and running.

#### Walkthrough

The following three-part guide showcases how to get everything ready for building a HOPR app.

- ["Running a local HOPR Cluster"](/developers/starting-local-cluster)
- ["Interacting with a HOPR node"](/developers/connecting-node)
- ["HOPR Apps - Hello world"](/developers/tutorial-hello-world)

#### OpenAPI Documentation

We use the [OpenAPI standard](https://swagger.io/specification/) to document our REST API. You can it in our
["REST API"](/developers/rest-api) section.

If you are running a hoprd node, you can see the actually exposed API endpoint of YOUR node at [http://localhost:3001/api/v2/\_swagger/](http://localhost:3001/api/v2/_swagger/)

### Visualizing topology of HOPR network

Before data packets can be sent between HOPR nodes, nodes need to possess **HOPR** tokens (or **tHOPR** for testnets) and open payment channels with other nodes in the _HoprChannels_ smart contract. By monitoring opening, closing and updates of payment channels through events emitted from _HoprChannels_, the topology of the current HOPR network can be effectively mapped out.

#### HoprChannel Events

The following events are relevant for visualizing the network topology:

- `ChannelUpdated`
- `ChannelOpened`
- `ChannelFunded`
- `ChannelClosureInitiated`
- `ChannelClosureFinalized`

Connection (incluiding the direction of payment channels), stake (amount of HOPR tokens as channel balance), ticket redemption (number of tickets being redeemd per channel) and change change in channel status can be visualized with on-chain events.

A full specification of all the events from _HoprChannels_ smart contract is detailed in section ["Smart Contract Overview"](/developers/smart-contract)

#### Importance score

With the latest channel balances, **importance score** can be calculated per channel. This score is used as an indicator for cover traffic nodes to prioritize the distribution of cover traffic.
The importance score is calculated as a product of the **stake** of a node and the sum of **weights** of all the outgoing channels.

$$
\Omega(N_i) = st(N_i) \cdot \sum_{j} w(C_{N_iN_j})
$$

where $N_{i}$ is the node, $C_{N_iN_j}$ is an outgoing channel of node $N_{i}$, $w(C_{N_iN_j})$ is the weight of the channel $C_{N_iN_j}$.

$$
w(C_{N_iN_j}) = \sqrt{B(C_{N_iN_j}) \cdot \dfrac{St(N_j)}{St(N_i)}}
$$

where $B(C_{N_iN_j})$ is the balance of the channel between node $N_i$ and $N_j$, and $St(N_i)$ is the stake of the node $N_i$.

The **stake** of a node can be denoted as below, which is the sum of the unreleased tokens of a node $N_i$ and the total of outgoing channels of $N_i$

$$
St(N_i) = uT(N_i) + \sum_{j} B(C_{N_iN_j})
$$

#### Deployed Channel contracts

_HoprChannels_ smart contract of the last public testnet - "Wildhorn v2" is deployed on Gnosis Chain at [0xF69C45B4246FD91F17AB9851987c7F100e0273cF](https://blockscout.com/xdai/mainnet/address/0xF69C45B4246FD91F17AB9851987c7F100e0273cF/contracts).

#### Other statistics

For reference, [HOPR xDAI Testnet - Wildhorn v2 Dashboard](https://dune.xyz/hoprnet/HOPR-xDAI-Testnet-Wildhorn-v2) shows some statistics about this public testnet. The analysis is done with on-chain data of emitted events and transaction calls.

$$
$$
