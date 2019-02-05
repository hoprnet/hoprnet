'use strict'

const pull = require('pull-stream')
const { createHash } = require('crypto')
const lp = require('pull-length-prefixed')

const HASH_LENGTH = 16
const { PROTOCOL_HEARTBEAT } = require('../constants')

module.exports = (node) => node.handle(PROTOCOL_HEARTBEAT, (protocol, conn) => {
    pull(
        conn,
        lp.decode(),
        pull.filter(data => data.length === HASH_LENGTH),
        pull.asyncMap((data, cb) => {
            conn.getPeerInfo((err, peerInfo) => {
                if (err)
                    return cb(err)
                    
                node.peerBook.getAll()[peerInfo.id].lastSeen = Date.now()

                return cb(null, createHash('sha256').update(data).digest().slice(0, 16))

            })
        }),
        lp.encode(),
        conn
    )
})