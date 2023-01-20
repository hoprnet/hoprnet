---
id: network-registry
title: Network Registry
---

To test HOPR protocol and develop dApps on top of HOPR at a resonable scale, nodes are only allowed to join the network (sending messages) if they are registered on a "Network Registry" smart contract.

This restriction on the access guarded by the "Network Registry" is only enabled in the staging or production environment by default. If you are running a cluster of HOPR nodes locally in the anvil network, the "Network Registry" is not enabled.

There are two ways of registering a node:

- By the node runner itself, providing the node runner is eligible; or
- By the owner of the `HoprNetworkRegistry` smart contract

### Register a node by the runner

#### Eligibility

A node can be registered by its runner if the runner account is eligible. There are two ways to become an eligible account:

- A node runner's Ethereum account is staking in the HOPR stake program for a minimum stake of 1000 xHOPR token and one of the NFTs from the "allowed list" _(Currently disabled)_
- A node runner's Ethereum account is staking a "HOPR Boost NFT" of type `Network_registry`

To stake xHOPR tokens or "Network_registry" HoprBoost NFT, you can interact directly with the staking contract of the environment your HOPR node is running on. For production network, there is even a [web application](/staking/how-to-stake) for such a purpose.

#### Register the peer ID

An eligible node runner can call `selfRegister(string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to register one or more HOPR node. The number of nodes one account is allowed to register is subject to the `rank` of the "Network Registry" NFT the account has staked.

### Deregister a node

A node runner can call `selfDeregister(string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to remove previously registered HOPR nodes.

### Register a node by the Network Registry contract owner

#### Eligibility

Owner can register any account for any node. The eligibility of an account is not going to be checked unless a `sync` method for that account gets called.

#### Register the peer ID

Owner can call `ownerRegister(address[] accounts, string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to register a list of HOPR nodes for a list of accounts respectively. Note that this registration can overwrite existing entries.

### Deregister a node

Owner can call `ownerDeregister(string[] hoprPeerIds)` method from `HoprNetworkRegistry` smart contract to remove a list of nodes.
