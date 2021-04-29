# Introduction

The following are particular topics the team has agreed to discuss on, or are relevant for HOPR in the future. As the protocol is continously evolving, these are taken in consideration, but will not be considered into implementation until a proper proposal is created.

## Definition

_These discussion involve the actual definition of the HOPR protocol, as defined by our yellowpaper. Some topics within the protocol are external to its definition, and are instead better suited to be implemented as a separate discussion._

### Decover traffic allocation = % of staked + unreleased tokens

#### Problem Statement
Cover traffic should be allocated proportional to the total amount of (a) HOPR tokens staked + (b) unreleased tokens (for early token buyers & team).

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

