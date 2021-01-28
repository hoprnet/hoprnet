<!-- INTRODUCTION -->
<p align="center">
  <a href="https://hoprnet.org" target="_blank" rel="noopener noreferrer">
    <img width="100" src="https://github.com/hoprnet/hopr-assets/blob/master/v1/logo/hopr_logo_padded.png?raw=true" alt="HOPR Logo">
  </a>
  
  <!-- Title Placeholder -->
  <h3 align="center">HOPR</h3>
  <p align="center">
    <code>A project by the HOPR Association</code>
  </p>
  <p align="center">
    HOPR is a privacy-preserving messaging protocol which enables the creation of a secure communication network via relay nodes powered by economic incentives using digital tokens.
  </p>
</p>

# hopr-connect

**Disclaimer**: support for libp2p test suite WIP, see [KNOWN ISSUES](#known-issues)

[![](https://github.com/libp2p/js-libp2p-interfaces/raw/master/src/transport/img/badge.png)](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/transport)
[![](https://github.com/libp2p/js-libp2p-interfaces/raw/master/src/connection/img/badge.png)](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/connection)
[![](https://github.com/libp2p/js-libp2p-interfaces/raw/master/src/peer-discovery/img/badge.png)](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/peer-discovery)

## Description

A [transport module](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/transport) for [js-libp2p](https://github.com/libp2p/js-libp2p) that handles NAT traversal automatically by using peers in the network and without requiring external resources such as public STUN or TURN servers.

## Main features

- fully compatible with js-libp2p, see [KNOWN ISSUES](#known-issues)
- automatic usage of WebRTC:
  - try direct TCP connection, if not succesful
  - use any other available peer in the network as signalling server
  - perform WebRTC handshake(s)
  - upgrade to direct connection if possible
- use nodes in the network as STUN and TURN servers
- reconnect handling

## Usage

### Dependencies

- Node.js 12.x
- yarn

### Startup

Start a bootstrapServer

```ts
const libp2p = require('libp2p')
const MPLEX = require('libp2p-mplex')
const SECIO = require('libp2p-secio')
const PeerId = require('peer-id')

import HoprConnect from 'hopr-connect'
import Multiaddr from 'multiaddr'

const peerId = await PeerId.create({ keyType: 'secp256k1' })

const node = await libp2p.create({
  peerId,
  modules: {
    transport: [HoprConnect],
    streamMuxer: [MPLEX],
    connEncryption: [SECIO],
    peerDiscovery: [HoprConnect.discovery]
  },
  addresses: {
    listen: Multiaddr(`/ip4/127.0.0.1/tcp/9091/p2p/${peerId.toB58String()}`)
  }
})
```

Start another client

```ts
const libp2p = require('libp2p')
const MPLEX = require('libp2p-mplex')
const SECIO = require('libp2p-secio')
const PeerId = require('peer-id')

import HoprConnect from 'hopr-connect'
import Multiaddr from 'multiaddr'

const bootstrapId = '16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg' // Change this
const peerId = await PeerId.create({ keyType: 'secp256k1' })

const node = await libp2p.create({
  peerId
  modules: {
    transport: [HoprConnect],
    streamMuxer: [MPLEX],
    connEncryption: [SECIO],
    peerDiscovery: [HoprConnect.discovery]
  },
  addresses: {
    listen: Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${peerId.toB58String()}`)
  },
  config: {
    HoprConnect: {
      bootstrapServers: [Multiaddr(`/ip4/127.0.0.1/tcp/9091/p2p/${bootstrapId.toB58String()}`)],
      // Testing:
      __noDirectConnections: false, // set to true to simulate NAT
      __noWebRTCUpgrade: false // set to true to simulate bidirectional NAT
    }
  }
})
```

## Known issues

- IPv6 support disabled for the moment
- [WIP] libp2p test suite

## Contributors [in alphabetical order]

- [José Perez Aguinaga](https://github.com/jjperezaguinaga)
- [Peter Braden](https://github.com/peterbraden)
- [Sebastian Bürgel](https://github.com/scbuergel)
- [Robert Kiel](https://github.com/robertkiel)

and the rest of the [HOPR](https://hoprnet.org) team!
