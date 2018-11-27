'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')
const Multihash = require('multihashes')
const crypto = require('crypto')

const { PROTOCOL_DELIVER_PUBKEY } = require('./constants')

module.exports = (node) => (peerInfo, callback) => {
    function hasPublicKey(cb) {
        if (peerInfo.id.pubKey) {
            return callback(null, peerInfo.id.pubKey.marshal())
        } else if (node.peerBook.get(peerInfo.id).id.pubKey) {
            return callback(null, node.peerBook.get(peerInfo.id).id.pubKey.marshal())
        } else {
            cb(null, peerInfo)
        }
    }

    waterfall([
        (cb) => hasPublicKey(peerInfo, cb),
        (peerInfo, cb) => {
            if (!peerInfo.multiaddrs.size) {
                node.peerRouting.findPeer(peerInfo.id, cb)
            } else {
                cb(null, peerInfo)
            }
        },
        (peerInfo, cb) => hasPublicKey(peerInfo, cb),
        (peerInfo, cb) => node.dial(peerInfo, PROTOCOL_DELIVER_PUBKEY, cb),
        (conn, cb) => pull(
            conn,
            pull.drain(pubKey => waterfall([
                (cb) => conn.getPeerInfo(cb),
                (peerInfo, cb) => {
                    const multihash = Multihash.encode(crypto.createHash('sha256').update(pubKey).digest(), 'sha2-256')

                    if (peerInfo.id.id.compare(multihash) === 0) {
                        cb(null, pubKey)
                    } else {
                        cb(Error('General error.'))
                    }
                }
            ]))
        )
    ], callback)
}