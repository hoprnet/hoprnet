'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')

const { PROTOCOL_DELIVER_PUBKEY } = require('./constants')

module.exports = (node) => (peerInfo, callback) => waterfall([
    (cb) => {
        if (peerInfo.id.pubKey) {
            return callback(null, peerInfo.id.pubKey.marshal())
        } else if (node.peerBook.get(peerInfo.id).id.pubKey) {
            return callback(null, node.peerBook.get(peerInfo.id).id.pubKey.marshal())
        }

        if (!peerInfo.multiaddrs.size) {
            node.peerRouting.findPeer(peerInfo.id, cb)
        } else {
            cb(null, peerInfo)
        }   
    },
    (peerInfo, cb) => {
        if (peerInfo.id.pubKey) {
            return callback(null, peerInfo.id.pubKey.marshal())
        } else if (node.peerBook.get(peerInfo.id).id.pubKey) {
            return callback(null, node.peerBook.get(peerInfo.id).id.pubKey.marshal())
        }

        node.dialProtocol(peerInfo, PROTOCOL_DELIVER_PUBKEY, cb)
    },
    (conn, cb) => pull(
        conn,
        pull.drain(cb)
    )
], callback) 