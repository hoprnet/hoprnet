# Introduction

The following are particular topics the team has agreed to discuss on, or are relevant for HOPR in the future. As the protocol is continously evolving, these are taken in consideration, but will not be considered into implementation until a proper proposal is created.

## Definition

_These discussion involve the actual definition of the HOPR protocol, as defined by our yellowpaper. Some topics within the protocol are external to its definition, and are instead better suited to be implemented as a separate discussion._

### SURBs

#### Problem statement

Right now, because of sender/receiver unlinkability, the receiver can not know how to reply back to the sender.

#### Discussion

There exists a concept called SURBs - Single-Use Reply Blocks, This means, we create in addition to the header that we use to send the message to the receiver a second one that is used to reply to the sender.

We can refactor the header generation function such that we can create Reply-headers.

#### Additional notes

This was initially brought up in #743.

### Cover traffic allocation = % of staked + unreleased tokens

#### Problem statement

Cover traffic should be allocated proportional to the total amount of (a) HOPR tgokens staked + (b) unreleased tokens (for early token buyers & team).

#### Discussion

There's two design issues here:

- (a) how do we get total HOPR tokens staked by a node? e.g. sum of all amounts from funding events that the node paid minus the sum of all mounts from funding events which have a corresponding closing event. There's a few challenges here:

  - find all relevant events
  - assemble pub key from event
  - turn pub key into Ethereum address
  - find out if the key/address was partyA or B and who funded what

- (b) how do we link unreleased tokens that is assigned to an Ethereum address via the allocator contract (see #684) to a HOPR account of a node. This could be done via a linking smart contract that gets consumed here. Such a contract would be called for the beneficiary of the unreleased tokens (address noted in allocator contract) with a parameter of the HOPR pub key that it's assigning it's cover traffic to).

#### Additional notes

This was initially defined in #947.

### Incentives for low-level relaying

#### Problem statement

HOPRd uses `hopr-connect` to bypass NATs in order to connect to other nodes in the network and deliver messages. `hopr-connect` itself uses a combination of STUN- and TURN-like to bypass NATs. Both of these methods rely on work done by other nodes.

By default, every node offers these functionalities to all other nodes, without any direct compensation or countermeasure against over-usage.

Questions to be answered:

- Should nodes agree in relaying services for any other node?
- If there are any incentives, how and where should nodes be able to claim them?

## Implementation

_These discussions relate to the implementation of the HOPR protocol, or more particularly, `hoprd`. The protocol is defined by our yellowpaper, but the only existing implementation is located in `packages/hoprd`, as a Typescript/JavaScript node.js application._

### Use EIP712 for hashes before signing

#### Problem statement

HOPR signatures provide no context on what's being signed.

#### Discussion

By implementing [EIP-712](https://eips.ethereum.org/EIPS/eip-712), we can provide meaningful information to our hashes for further inspection and debugging.

#### Additional notes

Initially reported in #1365.

### Adding an on-ramp layer to hoprd

#### Problem statement

Right now we are stuck w/having to fund every initial node, which is cumbersome and not scalable for mainnet.

#### Discussion

Ideally, we provide a way within the node to allow users to be able to on-ramp their node directly in the app (e.g. integrate https://ramp.network/ as a command within chat or as a widget in hoprd)

#### Additional notes

Initially saw in #430.

### Switch from BN.js to BigNumber.js

#### Problem statement

Currently, HOPR is using BN.js and BigNumber.js . BN.js was introduced by Web3.js which got replaced by Ethers.js, hence HOPR is using multiple libraries to work with "big numbera" (32 bytes (Ethereum) rather 4 bytes (Javascript)).

Streamlining HOPR to use only library will make things easier and prevents from converting to one or the other representation of "big numbers".

## Ecosystem

_These discussion relate to topics that involve the HOPR ecosystem as a whole, either on how people interact with the protocol, `hoprd`, its documentantion, or other principles around privacy._

### Self-host documentation

#### Problem statement

Currently, we are hosting our [documentation](http://docs.hoprnet.org/en/latest/) on an online service, readthedocs.org.

#### Discussion

For increase security and privacy, we should be hosting the documentation ourselves and/or via a decentralized platform (e.g. IPFS)

## Premature Optimisations

At the moment we want to create a proof of concept, therefore any optimisation
is out of scope.

- Shrinking packet size by removing derivable values (initially in #1523)
- Bulk redeem tickets (originally #793)

## Tracking pending state

There is a delay between the time where we trigger an on-chain action and the time where the indexer registers it. We need to discuss how we are going to track _pending_ state.

## Ticket Value and Win Probability

At the moment these are global constants. At some point we want to adjust this.
One point of view is that they are constants that are voted on by the DAO.
Another is that they are variable and bidded on in a free market.
All of this is out of scope for now.
