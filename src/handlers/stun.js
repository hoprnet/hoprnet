'use strict'

const { PROTOCOL_STUN } = require('../constants')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

module.exports = (node) => node.handle(PROTOCOL_STUN, (protocol, conn) => {
    conn.getObservedAddrs((err, addrs) => {
        if (err)
            console.log('TODO: STUN error')

        console.log(`STUN: ${addrs}.`)

        pull(
            pull.values(addrs),
            lp.encode(),
            conn
        )
    })
})