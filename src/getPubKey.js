'use strict'

const pull = require('pull-stream')
const { waterfall, parallel } = require('async')
const PeerId = require('peer-id')

const { PROTOCOL_DELIVER_PUBKEY } = require('./constants')

// TODO add pubKey to peerBook
module.exports = (node) => (peerInfo, callback) => {
    function hasPublicKey(peerInfo, cb) {
        if (peerInfo.id.pubKey) {
            callback(null, peerInfo.id)
        } else if (node.peerBook.has(peerInfo.id) && node.peerBook.get(peerInfo.id).id.pubKey) {
            callback(null, node.peerBook.get(peerInfo.id).id)
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
        (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_DELIVER_PUBKEY, cb),
        (conn, cb) => parallel({
            peerId: (cb) => waterfall([
                (cb) => pull(conn, pull.drain((pubKey) => cb(null, pubKey))),
                (pubKey, cb) => PeerId.createFromPubKey(pubKey, cb)
            ], cb),
            peerInfo: (cb) => conn.getPeerInfo(cb)
        }, cb),
        ({ peerId, peerInfo }, cb) => {
            if (peerId.toBytes().compare(peerInfo.id.toBytes()) === 0) {
                cb(null, peerId)
            } else {
                cb(Error('General error.'))
            }
        }
    ], callback)
}