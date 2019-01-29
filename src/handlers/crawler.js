'use strict'

const pull = require('pull-stream')
const { waterfall } = require('neo-async')
const lp = require('pull-length-prefixed')


const { PROTOCOL_CRAWLING, CRAWLING_RESPONSE_NODES } = require('../constants')
const { randomSubset } = require('../utils')

module.exports = (node) => node.handle(PROTOCOL_CRAWLING, (protocol, conn) => waterfall([
    (cb) => conn.getPeerInfo(cb),
    (connectedPeerInfo, cb) => {
        const peers = node.peerBook.getAllArray()

        const selected = randomSubset(
            peers,
            Math.min(CRAWLING_RESPONSE_NODES, peers.length - 1),
            (peerInfo) => peerInfo.id.pubKey &&
                peerInfo.id.toBytes().compare(connectedPeerInfo.id.toBytes()) !== 0 &&
                peerInfo.id.toBytes().compare(node.peerInfo.id.toBytes()) !== 0
        ).map((peerInfo) => peerInfo.id.pubKey.bytes)

        pull(
            pull.values(selected),
            lp.encode(),
            conn
        )
    }
]))