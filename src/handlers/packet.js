'use strict'

const pull = require('pull-stream')
const paramap = require('pull-paramap')
const rlp = require('rlp')
const secp256k1 = require('secp256k1')
const Queue = require('promise-queue')

const lp = require('pull-length-prefixed')
const { log, getId, pubKeyToEthereumAddress } = require('../utils')

const { PROTOCOL_STRING } = require('../constants')
const Packet = require('../packet')
const Acknowledgement = require('../acknowledgement')

const PRIVATE_KEY_LENGTH = 32

module.exports = (node, options) => {
    // Registers the packet handlers if the node started as a
    // relay node.
    // This disables the relay functionality for bootstrap
    // nodes.
    if (options['bootstrap-node']) return

    async function forwardPacket(packet) {
        log(node.peerInfo.id, `Forwarding to node \x1b[34m${packet._targetPeerId.toB58String()}\x1b[0m.`)

        const conn = await Promise.race([
            node.peerRouting.findPeer(packet._targetPeerId).then(peerInfo => node.dialProtocol(peerInfo, PROTOCOL_STRING)),
            node.dialProtocol(packet._targetPeerId, PROTOCOL_STRING)
        ])

        pull(
            pull.once(packet.toBuffer()),
            lp.encode(),
            conn,
            lp.decode({
                maxLength: Acknowledgement.SIZE
            }),
            pull.drain(data => {
                if (data.length != Acknowledgement.SIZE) return

                handleAcknowledgement(Acknowledgement.fromBuffer(data))
            })
        )
    }

    async function handleAcknowledgement(ack) {
        if (!ack.challengeSigningParty.equals(node.peerInfo.id.pubKey.marshal())) {
            console.log(
                `peer ${node.peerInfo.id.toB58String()} channelId ${getId(
                    pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                    pubKeyToEthereumAddress(ack.responseSigningParty)
                ).toString('hex')}`
            )

            return node.paymentChannels.contractCall(
                node.paymentChannels.contract.methods.wrongAcknowledgement(
                    ack.challengeSignature.slice(0, 32),
                    ack.challengeSignature.slice(32, 64),
                    ack.responseSignature.slice(0, 32),
                    ack.responseSignature.slice(32, 64),
                    ack.key,
                    ack.challengeSignatureRecovery,
                    ack.responseSignatureRecovery
                ),
                (err, receipt) => {
                    console.log(err, receipt)
                }
            )
        }

        let channelId
        try {
            channelId = await node.db.get(node.paymentChannels.ChannelId(ack.challengeSignatureHash))
        } catch (err) {
            if (err.notFound) return

            throw err
        }

        const challenge = secp256k1.publicKeyCreate(ack.key)
        const ownKeyHalf = await node.db.get(node.paymentChannels.Challenge(channelId, challenge))

        let channelKey
        try {
            channelKey = await node.db.get(node.paymentChannels.ChannelKey(channelId))
        } catch (err) {
            if (err.notFound) {
                channelKey = Buffer.alloc(PRIVATE_KEY_LENGTH, 0)
            } else {
                throw err
            }
        }

        node.db
            .batch()
            .put(node.paymentChannels.ChannelKey(channelId), secp256k1.privateKeyTweakAdd(channelKey, secp256k1.privateKeyTweakAdd(ack.key, ownKeyHalf)))
            .del(node.paymentChannels.Challenge(channelId, challenge))
            .write()
    }

    const queues = new Map()

    node.handle(PROTOCOL_STRING, (protocol, conn) =>
        pull(
            conn,
            lp.decode({
                maxLength: Packet.SIZE
            }),
            paramap(async (data, cb) => {
                const packet = Packet.fromBuffer(data)

                const sender = await packet.getSenderPeerId()

                const channelId = getId(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), pubKeyToEthereumAddress(sender.pubKey.marshal()))

                let queue = queues.get(channelId.toString('base64'))
                if (!queue) {
                    queue = new Queue(1, Infinity)
                    queues.set(channelId.toString('base64'), queue)
                }

                queue.add(() =>
                    packet
                        .forwardTransform(node)
                        .then(packet => {
                            if (node.peerInfo.id.isEqual(packet._targetPeerId)) {
                                options.output(demo(packet.message.plaintext))
                            } else {
                                forwardPacket(packet)
                            }

                            return cb(null, Acknowledgement.create(packet.oldChallenge, packet.header.derivedSecret, node.peerInfo.id).toBuffer())
                        })
                        .catch(err => {
                            console.log(err)
                            return cb(null, Buffer.alloc(0))
                        })
                )
            }),
            lp.encode(),
            conn
        )
    )

    function demo(plaintext) {
        const message = rlp.decode(plaintext)

        return `\n\n---------- New Message ----------\nMessage "${message[0].toString()}" latency ${Date.now() -
            Number(message[1].toString())} ms.\n---------------------------------\n\n`
    }
}
