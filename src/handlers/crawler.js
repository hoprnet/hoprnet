'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const { PROTOCOL_CRAWLING, CRAWLING_RESPONSE_NODES } = require('../constants')
const { randomSubset } = require('../utils')

module.exports = (node) => node.handle(PROTOCOL_CRAWLING, (protocol, conn) => {
    const peers = node.peerBook.getAllArray()

    const filter = (peerInfo) =>
        peerInfo.id.pubKey &&
        // peerInfo.id.toBytes().compare(connectedPeerInfo.id.toBytes()) !== 0 &&
        peerInfo.id.toBytes().compare(node.peerInfo.id.toBytes()) !== 0

    const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, peers.length - 1)

    const selectedNodes = randomSubset(peers, amountOfNodes, filter)
        .map((peerInfo) => peerInfo.id.pubKey.bytes)

    pull(
        pull.values(selectedNodes),
        lp.encode(),
        conn
    )
})