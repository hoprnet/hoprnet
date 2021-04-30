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

## Ecosystem

_These discussion relate to topics that involve the HOPR ecosystem as a whole, either on how people interact with the protocol, `hoprd`, its documentantion, or other principles around privacy._

### Self-host documentation

#### Problem statement

Currently, we are hosting our [documentation](http://docs.hoprnet.org/en/latest/) on an online service, readthedocs.org.

#### Discussion

For increase security and privacy, we should be hosting the documentation ourselves and/or via a decentralized platform (e.g. IPFS)
