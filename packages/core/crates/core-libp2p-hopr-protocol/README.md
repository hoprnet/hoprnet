# libp2p-hopr-protocol
The package represents the network behavior known as HOPR network protocol, whereby a HOPR message representing encrypted data is transmitted within a mixnet to a specific peer that provides an acknowledgement mechanism confirming the message receipt.

The network behavior protocol is defined in terms of [`rust-libp2p`](https://github.com/libp2p/rust-libp2p)'s trait [`libp2p_swarm::NetworkBehaviour`](https://docs.rs/libp2p/latest/libp2p/swarm/trait.NetworkBehaviour.html) implementation.

