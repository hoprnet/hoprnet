'use strict'

const Multiaddr = require('multiaddr')
const Address4 = require('ip-address').Address4

const groupBy = require('lodash.groupby')

module.exports = class UDP4 {
    constructor(opts) {
        this.node = opts.libp2p

        this.tag = 'WebRTCv4'
        this.socketType = 'udp4'
        this.family = 'ipv4'
    }

    getLocalhost(serverAddr) {
        return Multiaddr.fromNodeAddress({ port: serverAddr.port, address: '127.0.0.1', family: 'IPv4' }, 'udp').encapsulate(
            `/ipfs/${this.node.peerInfo.id.toB58String()}`
        )
    }

    sortAddrs(multiaddrs) {
        const addrs = groupBy(multiaddrs, ma => {
            const addr = new Address4(ma.toOptions().host)

            if (addr.isInSubnet(new Address4('127.0.0.1/8'))) return 'loopback'

            if (
                addr.isInSubnet(new Address4('10.0.0.0/8')) ||
                addr.isInSubnet(new Address4('172.16.0.0/12')) ||
                addr.isInSubnet(new Address4('192.168.0.0/16'))
            )
                return 'local'

            if (addr.isInSubnet(new Address4('169.254.0.0/16'))) return 'link-local'

            return 'global'
        })

        const result = []

        // 1. globals
        // 2. locals
        // 3. localhost
        if (addrs.global) result.push(...addrs.global)
        if (addrs.local) result.push(...addrs.local)
        if (addrs.loopback) result.push(...addrs.loopback)

        return result
    }
}
