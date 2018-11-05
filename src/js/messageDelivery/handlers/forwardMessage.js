'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')
const parallel = require('async/parallel')

const c = require('../constants')
const Packet = require('../packet')
const Acknowledgement = require('../../payments/acknowledgement')


module.exports = (node, output) => node.handle(c.PROTOCOL_STRING, (protocol, conn) => {
    pull(
        conn,
        pull.filter(data =>
            data.length > 0 && Packet.SIZE === data.length
        ),
        pull.map(data => Packet.fromBuffer(data)),
        pull.drain(packet => waterfall([
            (cb) => conn.getPeerInfo(cb),
            (peerInfo, cb) => {
                packet.forwardTransform(node, peerInfo.id.pubKey.marshal(), (err, packet) => cb(err, packet, peerInfo))
            },
            (packet, peerInfo, cb) => parallel([
                (cb) => {
                    const targetPeerId = packet.getTargetPeerId()
                    if (node.peerInfo.id.toBytes().compare(targetPeerId.toBytes()) === 0) {
                        output(packet.message.plaintext.toString())
                    } else {
                        waterfall([
                            (cb) => node.peerRouting.findPeer(targetPeerId, cb),
                            (peerInfo, cb) => parallel({
                                transaction: (cb) => packet.addTransaction(peerInfo.id, node, cb),
                                conn: (cb) => node.dialProtocol(peerInfo, c.PROTOCOL_STRING, cb)
                            }, cb),
                            (results, cb) => {
                                pull(
                                    pull.once(packet.toBuffer()),
                                    results.conn
                                )
                                cb(null)
                            }
                        ], cb)
                    }
                },
                // (cb) => waterfall([
                //     (cb) => node.dialProtocol(peerInfo, c.PROTOCOL_ACKNOWLEDGEMENT, cb),
                //     (conn, cb) => {
                //         pull(
                //             pull.once(
                //                 Acknowledgement.create(
                //                     packet.challenge,
                //                     packet.header.derivedSecret,
                //                     node.peerInfo.id.privKey.marshal()
                //                 )
                //             ),
                //             conn
                //         )
                //         cb()
                //     }
                // ], cb)
            ], cb)
        ], console.log))
    )
})