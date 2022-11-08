---
id: visualising-hopr-network-topology
title: Visualizing topology of HOPR network
---

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

In the protocol, the **importance score** is implemented as the function `importance` in the [packages/cover-traffic-daemon/src/utils.ts](https://github.com/hoprnet/hoprnet/blob/master/packages/cover-traffic-daemon/src/utils.ts)

#### Deployed Channel contracts

_HoprChannels_ smart contract of the previous public testnet - "Wildhorn v2" is deployed on Gnosis Chain at [0xF69C45B4246FD91F17AB9851987c7F100e0273cF](https://blockscout.com/xdai/mainnet/address/0xF69C45B4246FD91F17AB9851987c7F100e0273cF/contracts).

_HoprChannels_ smart contract of the last public testnet - "Matterhorn" is deployed on Gnosis Chain at [0xD2F008718EEdD7aF7E9a466F5D68bb77D03B8F7A](https://blockscout.com/xdai/mainnet/address/0xD2F008718EEdD7aF7E9a466F5D68bb77D03B8F7A/transactions).

#### Other statistics

For reference, [HOPR xDAI Testnet - Wildhorn v2 Dashboard](https://dune.xyz/hoprnet/HOPR-xDAI-Testnet-Wildhorn-v2) shows some statistics about this public testnet. The analysis is done with on-chain data of emitted events and transaction calls.

The [HOPR protocol - xDAI: Matterhorn Testnet](https://dune.xyz/hoprnet/HOPR-xDAI-Testnet-Matterhorn) shows the recently completed public testnet. Read more on the retrospective of this testnet in the [medium blog](https://medium.com/hoprnet/matterhorn-retrospective-c37f0077b13e).
