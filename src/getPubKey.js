'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { tryEach, waterfall } = require('neo-async')
const PeerId = require('peer-id')
const PeerInfo = require('peer-info')

const { PROTOCOL_DELIVER_PUBKEY } = require('./constants')
const { pubKeyToPeerId } = require('./utils')

const COMPRESSED_PUBLIC_KEY_SIZE = 33

module.exports = (node) => {
    const queryNode = (peerId, cb) =>
        tryEach([
            (cb) => node.dialProtocol(new PeerInfo(peerId), PROTOCOL_DELIVER_PUBKEY, cb),
            (cb) => waterfall([
                (cb) => node.peerRouting.findPeer(peerId, cb),
                (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_DELIVER_PUBKEY, cb)
            ], cb)
        ], (err, conn) => {
            if (node.peerBook.has(targetPeerInfo.id) && node.peerBook.get(targetPeerInfo.id).id.pubKey)
                return cb(null, node.peerBook.get(peerId))

            if (err || !conn)
                return cb(err)

            try {
                pull(
                    conn,
                    lp.decode({
                        maxLength: COMPRESSED_PUBLIC_KEY_SIZE
                    }),
                    pull.collect((err, pubKeys) => {
                        if (err || pubKeys.length != 1 || pubKeys[0].length != COMPRESSED_PUBLIC_KEY_SIZE)
                            return cb(Error(`Invalid response from ${peerId.toB58String()}`))

                        const peerInfo = new PeerInfo(pubKeyToPeerId(pubKeys[0]))
                        peerInfo.multiaddrs.replace([], targetPeerInfo.multiaddrs.toArray())
                        node.peerBook.put(peerInfo)

                        cb(null, peerInfo)
                    })
                )
            } catch (err) {
                return cb(err)
            }
        })

    return (peer, callback) => {
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

        queryNode(peer, callback)
    }
}