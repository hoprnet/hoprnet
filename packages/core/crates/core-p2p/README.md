# P2P

The underlying technology for managing the peer-to-peer networking used by this package is the [`rust-libp2p`](https://github.com/libp2p/rust-libp2p) library ([documentation](https://docs.libp2p.io/)).

## Modularity

`rust-libp2p` is highly modular allowing for reimplmenting expected behavior using custom implementations for API traits.

This way it is possible to experiment with and combine different components of the library in order to construct a specific targeted use case.

## `rust-libp2p` connectivity

As per the [official documentation](https://connectivity.libp2p.io/), the connectivity types in the library are divided into the `standalone` (implementation of network over host) and `browser` (implementation of network over browser).

Nodes that are not located behind a blocking firewall or NAT are designated as **public nodes** and can utilize the `TCP` or `QUIC` connectivity, with the recommendation to use QUIC if possible.

Browser based solutions are almost always located behind a private network or a blocking firewall and to open a connection towards the standalone nodes these utilize either the `WebSocket` approach (by hijacking the `TCP` connection) or the (not yet fully speced up) `WebTransport` (by hijacking the `QUIC` connection).

`WebRTC` is used to allow both outgoing and incoming connections to both standalone and browser based nodes.

## WASM

Due to the current project layout and future options the WASM compatibility of the used implementation is necessary. This package:

1. re-implements the `TCP` connection using JavaScript
2. implements the `Transport` trait for the JavaScript enabled WASM object
3. makes the building blocks for the libp2p communication compatible with the [`libp2p-wasm-ext`](https://crates.io/crates/libp2p-wasm-ext) module

### Specifics

- In order to implement a connectivity mechanism for the `libp2p-wasm-ext` module, it must satisfy the [`ffi` interface](https://docs.rs/libp2p-wasm-ext/latest/libp2p_wasm_ext/ffi/index.html).

- The reference implementation for a browser implementation of the websocket communication is provided in the modules ([here](https://docs.rs/libp2p-wasm-ext/latest/libp2p_wasm_ext/ffi/fn.websocket_transport.html)).

- The custom implementations for the WASM environment is heavily inspired by the example [reference](https://github.com/libp2p/rust-libp2p/blob/1a9cf4f7760724032b729c43165716c7ecd842ad/transports/wasm-ext/src/websockets.js).
