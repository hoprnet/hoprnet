'use strict'

const { PROTOCOL_STUN, NAME } = require('../constants')
const { parallel } = require('neo-async')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

module.exports = (node) => node.handle(PROTOCOL_STUN, (protocol, conn) =>
    parallel({
        addrs: (cb) => conn.getObservedAddrs(cb),
        peerInfo: (cb) => conn.getPeerInfo(cb)
    }, (err, { addrs, peerInfo }) => {
        if (err)
            console.log('TODO: STUN error')

        // peerInfo.multiaddrs.forEach((addr) => {
        //     if (addr.toOptions().host == '127.0.0.1') {
        //         peerInfo.multiaddrs.delete(addr)
        //     }
        // })
        
        addrs.forEach((addr) => peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`)))

        node.peerBook.put(peerInfo)

        console.log(`STUN: ${addrs.map(addr => addr.toString()).join(', ')}.`)
        pull(
            pull.values(addrs.map((addr) => addr.buffer)),
            lp.encode(),
            conn
        )
    })
)