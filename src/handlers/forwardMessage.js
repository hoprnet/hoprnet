'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { waterfall, parallel } = require('neo-async')
const { log } = require('../utils')

const c = require('../constants')
const Packet = require('../packet')
const Acknowledgement = require('../acknowledgement')

module.exports = (node, output) => node.handle(c.PROTOCOL_STRING, (protocol, conn) => {
    function forwardPacket(packet, cb) {
        const targetPeerId = packet.getTargetPeerId()
        if (node.peerInfo.id.toBytes().compare(targetPeerId.toBytes()) === 0) {
            output(demo(packet.message.plaintext.toString()))
        } else {
            log(node.peerInfo.id, `Forwarding to node \x1b[34m${targetPeerId.toB58String()}\x1b[0m.`)

            waterfall([
                (cb) => node.peerRouting.findPeer(targetPeerId, cb),
                (targetPeerInfo, cb) => node.dialProtocol(targetPeerInfo, c.PROTOCOL_STRING, cb),
                (conn, cb) => {
                    pull(
                        pull.once(packet.toBuffer()),
                        lp.encode(),
                        conn
                    )
                    cb(null)
                }
            ], cb)
        }
    }

    function sendAcknowledgement(packet, peerInfo, cb) {
        log(node.peerInfo.id, `Acknowledging to node \x1b[34m${peerInfo.id.toB58String()}\x1b[0m.`)
        waterfall([
            (cb) => node.dialProtocol(peerInfo, c.PROTOCOL_ACKNOWLEDGEMENT, cb),
            (conn, cb) => {
                pull(
                    pull.once(
                        Acknowledgement.create(
                            packet.oldChallenge,
                            packet.header.derivedSecret,
                            node.peerInfo.id
                        ).toBuffer()
                    ),
                    lp.encode(),
                    conn
                )
                cb()
            }
        ], cb)
    }

    pull(
        conn,
        lp.decode(),
        pull.filter(data => data.length > 0 && data.length % Packet.SIZE === 0),
        pull.map(data => Packet.fromBuffer(data)),
        pull.drain((packet) => waterfall([
            (cb) => conn.getPeerInfo(cb),
            (peerInfo, cb) => node.getPubKey(peerInfo, cb),
            (senderPeerInfo, cb) => packet.forwardTransform(node, senderPeerInfo.id, (err, packet) => cb(err, packet, senderPeerInfo)),
            (packet, senderPeerInfo, cb) => parallel([
                (cb) => forwardPacket(packet, cb),
                (cb) => sendAcknowledgement(packet, senderPeerInfo, cb)
            ], cb)
        ]))
    )
})

function demo(str) {
    const chunks = str.split(' ')

    return '\n\n---------- New Message ----------\nMessage \"' + chunks[0] + '\" latency ' + (Date.now() - Number(chunks[1])) + ' ms.\n---------------------------------\n\n'
}