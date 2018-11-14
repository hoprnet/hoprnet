'use strict'

const pull = require('pull-stream')

const { PROTOCOL_DELIVER_PUBKEY } = require('../constants')

module.exports = (node) => node.handle(PROTOCOL_DELIVER_PUBKEY, (protocol, conn) => {
    pull(
        pull.once(node.peerInfo.id.pubKey.marshal),
        conn
    )
})