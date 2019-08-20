'use strict'

const Multiaddr = require('multiaddr')
const Address6 = require('ip-address').Address6

const groupBy = require('lodash.groupby')

module.exports = class UDP6 {
    constructor(opts) {
        this.node = opts.libp2p

        this.tag = 'WebRTCv6'
        this.socketType = 'udp6'
        this.family = 'ipv6'
    }

    getLocalhost(serverAddr) {
        return Multiaddr.fromNodeAddress({ port: serverAddr.port, address: '::1', family: 'IPv6' }, 'udp').encapsulate(
            `/ipfs/${this.node.peerInfo.id.toB58String()}`
        )
    }

    sortAddrs(multiaddrs) {
        const addrs = groupBy(multiaddrs, ma => {
            const addr = new Address6(ma.toOptions().host)

            if (addr.isLoopback()) return 'loopback'

            if (addr.isLinkLocal()) return 'link-local'

            return 'global'
        })

        const result = []

        // 1. globals
        // 2. locals
        // 3. localhost
        if (addrs.global) result.push(...addrs.global)
        if (addrs['link-local']) result.push(...addrs['link-local'])
        if (addrs.loopback) result.push(...addrs.loopback)

        return result
    }
}
