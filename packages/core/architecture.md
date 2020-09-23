# Architecture

NB. This is a work in progress to document how and why the code is structured in
the way it is.

## Networking

### Transport

See `./src/network/transport`

We use a [custom transport layer](https://github.com/libp2p/interface-transport)
with libp2p.

It is a mixture of [TCP](https://github.com/libp2p/js-libp2p-tcp) and
[webRTC star](https://github.com/libp2p/js-libp2p-webrtc-star) with a
peer-to-peer structure (any peer can act as a signalling relay).

This is necessary as we have the following constraints:

1. We want anyone to be able to run a node, which means we need to be able to
   run behind NAT, and various firewalls.
2. We want to be truly decentralized, and not rely on any 'second party'
   signalling servers etc.

A lot of the complexity in this area of the code derives from the many different
coding styles used in the various libraries we must interact with, namely event
streams and asynchronous generator functions.

@TODO Questions:

- What are the reasons not to use multiple libp2p transports and upgrade with a
  libp2p switch?

#### Listener

Listener maintains a TCP and UDP socket and will listen to specified multiaddrs.
It will trigger handler when messages arrive.
