'use strict'

const pull = require('pull-stream')
const paramap = require('pull-paramap')
const rlp = require('rlp')
const { waterfall } = require('neo-async')

const lp = require('pull-length-prefixed')
const { log, getId, pubKeyToEthereumAddress } = require('../utils')

const { PROTOCOL_STRING } = require('../constants')
const Packet = require('../packet')
const Acknowledgement = require('../acknowledgement')

module.exports = (node, options) => {
    // Registers the packet handlers if the node started as a
    // relay node.
    // This disables the relay functionality for bootstrap
    // nodes.
    if (options['bootstrap-node'])
        return

    function forwardPacket(packet) {
        log(node.peerInfo.id, `Forwarding to node \x1b[34m${packet._targetPeerId.toB58String()}\x1b[0m.`)

        waterfall([
            (cb) => node.peerRouting.findPeer(packet._targetPeerId, cb),
            (targetPeerInfo, cb) => node.dialProtocol(targetPeerInfo, PROTOCOL_STRING, cb)
        ], (err, conn) => {
            if (err) {
                console.log(err)
                return
            }

            pull(
                pull.once(packet.toBuffer()),
                lp.encode(),
                conn,
                lp.decode({ maxLength: Acknowledgement.SIZE }),
                pull.drain((data) => {
                    if (data.length != Acknowledgement.SIZE)
                        return

                    handleAcknowledgement(Acknowledgement.fromBuffer(data))
                })
            )
        })
    }

    function handleAcknowledgement(ack) {
        if (!ack.challengeSigningParty.equals(node.peerInfo.id.pubKey.marshal())) {
            console.log(`peer ${node.peerInfo.id.toB58String()} channelId ${getId(
                pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(ack.responseSigningParty)
            ).toString('hex')}`)
            return node.paymentChannels.contractCall(node.paymentChannels.contract.methods.wrongAcknowledgement(
                ack.challengeSignature.slice(0, 32),
                ack.challengeSignature.slice(32, 64),
                ack.responseSignature.slice(0, 32),
                ack.responseSignature.slice(32, 64),
                ack.key,
                ack.challengeSignatureRecovery,
                ack.responseSignatureRecovery
            ), (err, receipt) => {
                console.log(err, receipt)
            })
        }

        waterfall([
            (cb) => node.paymentChannels.getChannelIdFromSignatureHash(ack.challengeSignatureHash, cb),
            (channelId, cb) => node.paymentChannels.solveChallenge(channelId, ack.key, cb)
        ], (err) => {
            if (err)
                throw err
        })
    }

    node.handle(PROTOCOL_STRING, (protocol, conn) =>
        pull(
            conn,
            lp.decode({
                maxLength: Packet.SIZE
            }),
            paramap((data, cb) => {
                const packet = Packet.fromBuffer(data)

                packet.forwardTransform(node, (err) => {
                    if (err) {
                        log(node.peerInfo.id, err.message)
                        return cb(null, Buffer.alloc(0))
                    }

                    if (node.peerInfo.id.isEqual(packet._targetPeerId)) {
                        options.output(demo(packet.message.plaintext))
                    } else {
                        forwardPacket(packet)
                    }

                    return cb(null, Acknowledgement.create(
                        packet.oldChallenge,
                        packet.header.derivedSecret,
                        node.peerInfo.id,
                    ).toBuffer())
                })

            }),
            lp.encode(),
            conn
        )
    )

    function demo(plaintext) {
        const message = rlp.decode(plaintext)

        return `\n\n---------- New Message ----------\nMessage "${message[0].toString()}" latency ${Date.now() - Number(message[1].toString())} ms.\n---------------------------------\n\n`
    }
}
