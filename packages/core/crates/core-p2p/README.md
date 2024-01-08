# P2P

The underlying technology for managing the peer-to-peer networking used by this package is the [`rust-libp2p`](https://github.com/libp2p/rust-libp2p) library ([documentation](https://docs.libp2p.io/)).

## Modularity

`rust-libp2p` is highly modular allowing for reimplmenting expected behavior using custom implementations for API traits.

This way it is possible to experiment with and combine different components of the library in order to construct a specific targeted use case.

## `rust-libp2p` connectivity

As per the [official documentation](https://connectivity.libp2p.io/), the connectivity types in the library are divided into the `standalone` (implementation of network over host) and `browser` (implementation of network over browser).

Nodes that are not located behind a blocking firewall or NAT are designated as **public nodes** and can utilize the `TCP` or `QUIC` connectivity, with the recommendation to use QUIC if possible.

Browser based solutions are almost always located behind a private network or a blocking firewall and to open a connection towards the standalone nodes these utilize either the `WebSocket` approach (by hijacking the `TCP` connection) or the (not yet fully speced up) `WebTransport` (by hijacking the `QUIC` connection).
