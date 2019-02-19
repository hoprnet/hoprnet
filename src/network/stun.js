'use strict'

const { PROTOCOL_STUN } = require('../constants')

const mafmt = require('mafmt')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

module.exports = (node, options) => (cb) => {
    // addr => tcp addrs
    node.dialProtocol(options.bootstrapServers[0], PROTOCOL_STUN, (err, conn) => 
        pull(
            conn,
            lp.decode(),
            pull.collect(cb)
        )
    )
}