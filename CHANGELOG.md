<a name="0.2.10"></a>

## [0.2.10](https://github.com/hoprnet/hopr-connect/compare/0.2.8...0.2.10) (2021-01-28)

### Breaking changes

#### Addressing

Before `hopr-connect@0.2.10`, the following addresses were valid:

- `Multiaddr("/ip4/127.0.0.1/tcp/0")`
- `Multiaddr("/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`
- `Multiaddr("/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`

Since `hopr-connect@0.2.10`, only addresses that include a PeerId are considered valid, namely:

- `Multiaddr("/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`
- `Multiaddr("/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`

### Fixes

- Always detect self-dial attempts

<a name="0.2.8"></a>

## [0.2.8](https://github.com/hoprnet/hopr-connect/compare/0.2.4...0.2.8) (2021-01-27)

### Fixes

- Various fixes
- Reduced console output

<a name="0.2.4"></a>

## [0.2.4](https://github.com/hoprnet/hopr-connect/compare/0.2.2...0.2.4) (2021-01-26)

### Fixes

- Prefix WebRTC stream to make sure it gets closed

<a name="0.2.2"></a>

## [0.2.2](https://github.com/hoprnet/hopr-connect/compare/0.2.1...0.2.2) (2021-01-25)

### Fixes

- Not removing WebRTC error listener to catch connection aborts

<a name="0.2.1"></a>

## [0.2.1](https://github.com/hoprnet/hopr-connect/compare/0.2.0...0.2.1) (2021-01-24)

### Fixes

- Control flow bug that lead to unintended connection closes

<a name="0.2.0"></a>

## [0.2.0](https://github.com/hoprnet/hopr-connect/compare/0.1.2...0.2.0) (2020-01-22)

### Enhancements

- Strong typing & less code
- Flexible upgrade handover sequence
- Priorisation of signalling messages over payload messages
- First integration of libp2p test suite

<a name="0.1.2"></a>

## [0.1.2](https://github.com/hoprnet/hopr-connect/compare/0.1.1...0.1.2) (2020-12-15)

### Fixes

- improved addressing and effective countermeasures against self-dials
- stronger typing
- various control-flow fixes

<a name="0.1.1"></a>

## [0.1.1](https://github.com/hoprnet/hopr-connect/compare/0.1...0.1.1) (2020-12-04)

### Fixes

- use `hopr-connect` in Debug strings

<a name="0.1"></a>

## [0.1](https://github.com/hoprnet/hopr-connect/compare/0.0.8...0.1) (2020-12-04)

### Features

- implements [PeerDiscovery](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/peer-discovery)
- type-checking for implemented interfaces, namely Connection, Transport, PeerDiscovery
- minor improvements

- resolve multiaddrs before dial ([#782](https://github.com/libp2p/js-libp2p/issues/782)) ([093c0ea](https://github.com/libp2p/js-libp2p/commit/093c0ea))

## Initial release

### Features

- automatic usage of WebRTC
- integration of STUN & TURN
- automatic handover between WebRTC and relayed connection
- proper handling of reconnects
