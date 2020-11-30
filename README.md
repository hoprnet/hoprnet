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

[![](https://github.com/libp2p/js-libp2p-interfaces/raw/master/src/transport/img/badge.png)](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/transport)
[![](https://github.com/libp2p/js-libp2p-interfaces/raw/master/src/connection/img/badge.png)](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/connection)
[![](https://github.com/libp2p/js-libp2p-interfaces/raw/master/src/peer-discovery/img/badge.png)](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/peer-discovery)

## Description

A [transport module](https://github.com/libp2p/js-libp2p-interfaces/tree/master/src/transport) for [js-libp2p](https://github.com/libp2p/js-libp2p) that handles NAT traversal automatically by using peers in the network and without requiring external resources such as public STUN or TURN servers.

## Main features

- fully compatible with js-libp2p
- automatic usage of WebRTC:
  - try direct TCP connection, if not succesful
  - use any other available peer in the network as signalling server
  - perform WebRTC handshake(s)
  - upgrade to direct connection if possible
- use nodes in the network as STUN and TURN servers
- proper reconnect handling
