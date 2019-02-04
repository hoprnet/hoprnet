'use strict'

const PacketHandler = require('./packet')
const Crawling = require('./crawler')
const Heartbeat = require('./heartbeat')
const DeliverPubKey = require('./deliverPubKey')

module.exports = (node, output) => {
    PacketHandler(node, output)
    Crawling(node)
    Heartbeat(node)

    // Disables the public key retrieval functionality
    // if the node was started without a public key, i. e.
    // as a bootstrap node.
    // if (node.peerInfo.id.pubKey) {
        // DeliverPubKey(node)
    // }
}