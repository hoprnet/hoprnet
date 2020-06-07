---
description: A technical description to HOPR.
---

# Technical Description

The construction consists of two layers: one for message delivery and one for payments. Messages are embedded within [SPHINX packet format](https://cypherpunks.ca/~iang/pubs/Sphinx_Oakland09.pdf) that provably hides the relation between sender and receiver. The payment layer uses off-chain payments via payment channels and node operators need to stake assets to process transactions.

- message delivery
  - [network layer](./Message-Delivery) establishes a peer-to-peer connection between the nodes. To achieve that, the implementation uses [libp2p](https://libp2p.io) in combination with [WebRTC](https://webrtc.org) to bypass NATs. This allows each node to become a relay node and earn money.
  - [messaging layer](./Packet-Format) hides the connection between sender and receiver of a message. Therefore it uses slightly modified version of the [SPHINX](https://cypherpunks.ca/~iang/pubs/Sphinx_Oakland09.pdf) packet format by G. Danezis and I. Goldberg.
- payment layer
  - [principle(s)](./Payment-Principles) explains the principles that are used to have incentivations that satisfy the stated security properties.
  - [protocol details](./Payment-Details) gives a deeper description how mechanims work.
