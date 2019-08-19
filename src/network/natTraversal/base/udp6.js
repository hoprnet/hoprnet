'use strict'

const Multiaddr = require('multiaddr')

module.exports = class UDP6 {
    constructor(opts) {
        this.node = opts.libp2p

        this.tag = 'WebRTCv6'
        this.socketType = 'udp6'
        this.family = 'ipv6'
    }

    getLocalhost(serverAddr) {
        return Multiaddr.fromNodeAddress({ port: serverAddr.port, address: 'fe80::1', family: 'IPv6' }, 'udp').encapsulate(`/ipfs/${this.node.peerInfo.id.toB58String()}`)
    }
}