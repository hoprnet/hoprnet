<a name="0.2.34"></a>

## [0.2.34](https://github.com/hoprnet/hopr-connect/compare/v0.2.33...v0.2.34) (2021-07-23)

Changes:

- dependency cleanup (#251)

<a name="0.2.33"></a>

## [0.2.33](https://github.com/hoprnet/hopr-connect/compare/v0.2.32...v0.2.33) (2021-07-22)

Changes:

- upgrade to `libp2p@0.32.1` (#249)
- fix automated relay CI test (#242)
- improved logging (#248)

<a name="0.2.32"></a>

## [0.2.32](https://github.com/hoprnet/hopr-connect/compare/v0.2.31...v0.2.32) (2021-07-21)

Changes:

- upgrade to `libp2p@0.32`

<a name="0.2.31"></a>

## [0.2.31](https://github.com/hoprnet/hopr-connect/compare/v0.2.30...v0.2.31) (2021-07-21)

### Breaking changes:

- Changed configuration object:

```ts
new HoprConnect(upgrader, {
  publicNodes, // EventEmitter
  initialNodes // Multiaddr[], list of already known nodes
})
```

The property `bootstrapNodes` is ignored.

### New features:

- support for relay slots, actively limitting maximum number of simultaneous relayed connections (#237)
- relay management API: dynamically add and remove potential relays and use them to bypass NATs (#243, #231)
- improved CI testing (#222, #227, #230, #232, #233, #234, #235)

### Fixes:

- fix `hangUp()` producing hanging promise (#240)

<a name="0.2.30"></a>

## [0.2.30](https://github.com/hoprnet/hopr-connect/compare/v0.2.29...v0.2.30) (2021-07-02)

Bugfix release

- see (#193) for improvements and bugfixes
- stricter testing
- improved socket tracking
- show meaningful message when connecting to wrong node
- show available addresses (#187)
- limit STUN request to reasonable amount (#185)
- package upgrades (#204)

<a name="0.2.29"></a>

## [0.2.29](https://github.com/hoprnet/hopr-connect/compare/v0.2.28...v0.2.29) (2021-06-15)

- fix STUN request failing on DNS failures
- minor bugfixes in STUN code

<a name="0.2.28"></a>

## [0.2.28](https://github.com/hoprnet/hopr-connect/compare/v0.2.27...v0.2.28) (2021-06-10)

- adds `node-pre-gyp` as dependency to make sure that `hopr-connect` includes all necessary dependencies for a standalone install
- fixes `wrtc` missing dependency `node-pre-gyp`

<a name="0.2.27"></a>

## [0.2.27](https://github.com/hoprnet/hopr-connect/compare/v0.2.26...v0.2.27) (2021-06-10)

- control-flow fixes
- remove console.log and console.trace

<a name="0.2.26"></a>

## [0.2.26](https://github.com/hoprnet/hopr-connect/compare/v0.2.25...v0.2.26) (2021-06-09)

- package upgrades
- correctly distinguish private and link-locale addresses from public addresses
- minor refactoring

<a name="0.2.25"></a>

## [0.2.25](https://github.com/hoprnet/hopr-connect/compare/v0.2.23...v0.2.25) (2021-05-11)

### Upgrades:

- `libp2p@0.31`
- `multiaddr@9.0`

<a name="0.2.23"></a>

## [0.2.23](https://github.com/hoprnet/hopr-connect/compare/v0.2.21...v0.2.23) (2021-03-01)

### Fixes

- close relayed connections if counterparty is not reachable

### Breaking changes

- Subnet detection: Nodes are able to detect whether the dialed address is a private address which is not in their reachable subnets.
- Nodes do not consider link-locale IP addresses as dialable

#### Addressing

Before `hopr-connect@0.2.22` the follwoing addresses were valid

- `Multiaddr("/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`
- `Multiaddr("/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`

Since `hopr-connect@0.2.22`, nodes need to explicitly name the relay behind which they are accessible

- `Multiaddr("/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`
- `Multiaddr("/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb/p2p-circuit/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg")`

Nodes will automatically dial given bootstrap addresses and check whether they can reach them and announce them ordered by latency to other nodes.

#### Default export to named export

Before `hopr-connect@0.2.23`, the main class HoprConnect was a default export, since `hopr-connect@0.2.23` it is a named export.

<a name="0.2.21"></a>

## [0.2.21](https://github.com/hoprnet/hopr-connect/compare/v0.2.20...v0.2.21) (2021-02-16)

### Fixes

- Fix STUN issues that prevent bootstap node from publishing public IPv4 addresses (#86)

<a name="0.2.20"></a>

## [0.2.20](https://github.com/hoprnet/hopr-connect/compare/0.2.12...v0.2.20) (2021-02-12)

### Fixes

- don't detect STUN timeouts as bidirectional NAT
- package upgrades

<a name="0.2.12"></a>

## [0.2.12](https://github.com/hoprnet/hopr-connect/compare/0.2.11...0.2.12) (2021-02-03)

### Fixes

- properly expose own TCP address as e.g. `/ip4/127.0.0.1/tcp/12345/p2p/<MyPeerId>`

### Changes

- Node.JS 12 -> Node.JS 14
- libp2p 0.29 -> Node.JS 0.30 (only for testing)
- libp2p-secio -> libp2p-noise (only for testing)

<a name="0.2.11"></a>

## [0.2.11](https://github.com/hoprnet/hopr-connect/compare/0.2.10...0.2.11) (2021-01-29)

### Fixes

- refactored internal communication
- less verbose debug output

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
