'use strict'

const Multiaddr = require('multiaddr')

module.exports = class UDP4 {
    constructor(opts) {
        this.node = opts.libp2p

        this.tag = 'WebRTCv4'
        this.socketType = 'udp4'
        this.family = 'ipv4'
    }

    getLocalhost(serverAddr) {
        return Multiaddr.fromNodeAddress({ port: serverAddr.port, address: '127.0.0.1', family: 'IPv4' }, 'udp').encapsulate(`/ipfs/${this.node.peerInfo.id.toB58String()}`)
    }
}
