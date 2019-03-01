'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { waterfall } = require('neo-async')
const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const { PROTOCOL_DELIVER_PUBKEY, MARSHALLED_PUBLIC_KEY_SIZE } = require('./constants')

module.exports = (node) => (peer, callback) => {
    if (PeerInfo.isPeerInfo(peer))
        peer = peer.id

    if (!PeerId.isPeerId(peer))
        callback(Error('Unable to convert input to peerId'))

    if (peer.pubKey)
        return callback(null, new PeerInfo(peer))

    if (node.peerBook.has(peer) && node.peerBook.get(peer).id.pubKey)
        return callback(null, node.peerBook.get(peer))

    if (peer.isEqual(node.peerInfo.id))
        return callback(null, node.peerInfo)

    waterfall([
        (cb) => node.peerRouting.findPeer(peer, cb),
        (targetPeerInfo, cb) => {
            if (targetPeerInfo.id.pubKey)
                return cb(null, targetPeerInfo)

            if (node.peerBook.has(targetPeerInfo.id) && node.peerBook.get(targetPeerInfo.id).id.pubKey)
                return cb(null, node.peerBook.get(peer))

            node.dialProtocol(targetPeerInfo, PROTOCOL_DELIVER_PUBKEY, (err, conn) => waterfall([
                (cb) => pull(
                    conn,
                    lp.decode({
                        maxLength: MARSHALLED_PUBLIC_KEY_SIZE
                    }),
                    pull.take(1),
                    pull.drain((data) => cb(null, data))
                ),
                (pubKey, cb) => PeerId.createFromPubKey(pubKey, cb),
                (peerId, cb) => {
                    const peerInfo = new PeerInfo(peerId)
                    peerInfo.multiaddrs.replace([], targetPeerInfo.multiaddrs.toArray())
                    cb(null, peerInfo)
                }
            ], cb))
        }
    ], callback)
}