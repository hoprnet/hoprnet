---
description: A technical description to HOPR.
---

# Technical Description

The construction consists of two layers: one for message delivery and one for payments. Messages are embedded within [SPHINX packet format](https://cypherpunks.ca/~iang/pubs/Sphinx_Oakland09.pdf) that provably hides the relation between sender and receiver. The payment layer uses off-chain payments via payment channels and node operators need to stake assets to process transactions.

- Message Delivery

  - [network layer](https://github.com/hoprnet/hopr-documentation/tree/a272b6a0eba46e804b16b5bd1d48ee19e950f101/main-concepts/Message-Delivery/README.md) establishes a peer-to-peer connection between the nodes. To achieve that, the implementation uses [libp2p](https://libp2p.io) in combination with [WebRTC](https://webrtc.org) to bypass NATs. This allows each node to become a relay node and earn money.
  - [messaging layer](https://github.com/hoprnet/hopr-documentation/tree/a272b6a0eba46e804b16b5bd1d48ee19e950f101/main-concepts/Packet-Format/README.md) hides the connection between sender and receiver of a message. Therefore it uses slightly modified version of the [SPHINX](https://cypherpunks.ca/~iang/pubs/Sphinx_Oakland09.pdf) packet format by G. Danezis and I. Goldberg.

- Payment Layer
  - [principle\(s\)](https://github.com/hoprnet/hopr-documentation/tree/a272b6a0eba46e804b16b5bd1d48ee19e950f101/main-concepts/Payment-Principles/README.md) explains the principles that are used to have incentivations that satisfy the stated security properties.
  - [protocol details](https://github.com/hoprnet/hopr-documentation/tree/a272b6a0eba46e804b16b5bd1d48ee19e950f101/main-concepts/Payment-Details/README.md) gives a deeper description how mechanims work.
