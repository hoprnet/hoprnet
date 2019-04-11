'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const { PROTOCOL_CRAWLING, CRAWLING_RESPONSE_NODES } = require('../constants')
const { randomSubset } = require('../utils')

module.exports = (node) => {
    const handler = (protocol, conn) => {
        const peers = node.peerBook.getAllArray()

        const filter = (peerInfo) =>
            peerInfo.id.pubKey &&
            !peerInfo.id.isEqual(node.peerInfo.id)

        const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, peers.length)

        const selectedNodes = randomSubset(peers, amountOfNodes, filter)
            .map((peerInfo) => peerInfo.id.pubKey.bytes)

        pull(
            pull.values(selectedNodes),
            lp.encode(),
            conn
        )
    }
    
    node.handle(PROTOCOL_CRAWLING, handler)
}