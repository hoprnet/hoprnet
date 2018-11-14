'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')

const { PROTOCOL_CRAWLING, MARSHALLED_PUBLIC_KEY_SIZE, CRAWLING_RESPONSE_NODES } = require('../constants')
const { randomSubset } = require('../utils')

module.exports = (node) => node.handle(PROTOCOL_CRAWLING, (protocol, conn) => waterfall([
    (cb) => conn.getPeerInfo(cb),
    (connectedPeerInfo, cb) => {
        const peers = node.peerBook.getAllArray()

        pull(
            pull.once(
                randomSubset(
                    peers,
                    Math.min(CRAWLING_RESPONSE_NODES, peers.length - 1),
                    (peerInfo) => peerInfo.id.pubKey &&
                        peerInfo.id.toBytes().compare(connectedPeerInfo.id.toBytes()) !== 0 &&
                        peerInfo.id.toBytes().compare(node.peerInfo.id.toBytes()) !== 0
                ).reduce(
                    (acc, peerInfo, index) =>
                        // TODO: Insert multiaddrs to decrease amount of roundtrips
                        Buffer.concat([acc, peerInfo.id.pubKey.bytes], (index + 1) * MARSHALLED_PUBLIC_KEY_SIZE),
                    Buffer.alloc(0))
            ),
            conn
        )
    }
]))