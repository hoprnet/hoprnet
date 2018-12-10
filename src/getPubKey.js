'use strict'

const pull = require('pull-stream')
const { waterfall } = require('async')
const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const { PROTOCOL_DELIVER_PUBKEY, MARSHALLED_PUBLIC_KEY_SIZE } = require('./constants')

// TODO add pubKey to peerBook
module.exports = (node) => (peerInfo, callback) => {
    if (peerInfo.id.pubKey) {
        callback(null, peerInfo)
    } else if (node.peerBook.has(peerInfo.id) && node.peerBook.get(peerInfo.id).id.pubKey) {
        callback(null, node.peerBook.get(peerInfo.id))
    } else if (peerInfo.id.toBytes().compare(node.peerInfo.id.toBytes()) === 0) {
        callback(null, node.peerInfo)
    } else {
        waterfall([
            (cb) => node.peerRouting.findPeer(peerInfo.id, cb),
            (targetPeerInfo, cb) => {
                if (targetPeerInfo.id.pubKey) {
                    cb(null, targetPeerInfo)
                } else if (node.peerBook.has(targetPeerInfo.id) && node.peerBook.get(targetPeerInfo.id).id.pubKey) {
                    cb(null, node.peerBook.get(peerInfo.id))
                } else {
                    node.dialProtocol(targetPeerInfo, PROTOCOL_DELIVER_PUBKEY, (err, conn) => waterfall([
                        (cb) => pull(
                            conn,
                            pull.filter((data) => data.length > 0 && data.length === MARSHALLED_PUBLIC_KEY_SIZE),
                            pull.collect(cb)
                        ),
                        (data, cb) => {
                            if (data.length !== 1)
                                cb(Error('Invalid response'))
    
                            PeerId.createFromPubKey(data[0], cb)
                        },
                        (peerId, cb) => PeerInfo.create(peerId, cb),
                        (peerInfo, cb) => {
                            targetPeerInfo.multiaddrs.forEach((addr) => {
                                peerInfo.multiaddrs.add(addr)
                            })
                            cb(null, peerInfo)
                        }
                    ], cb))
                }
            }], callback)
    }
}