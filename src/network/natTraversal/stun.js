'use strict'

const { PROTOCOL_STUN } = require('../../constants')

const mafmt = require('mafmt')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const Multiaddr = require('multiaddr')

module.exports = (node, options) => (cb) =>
    // addr => tcp addrs
    node.dialProtocol(options.bootstrapServers[0], PROTOCOL_STUN, (err, conn) => {
        if (err)
            return cb(err)

        pull(
            conn,
            lp.decode(),
            pull.map(data => Multiaddr(data)),
            pull.collect(cb)
        )

    })
