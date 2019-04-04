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
                lp.decode({
                    maxLength: Acknowledgement.SIZE
                }),
                pull.drain((data) => {
                    if (data.length != Acknowledgement.SIZE)
                        return

                    handleAcknowledgement(Acknowledgement.fromBuffer(data))
                })
            )
        })
    }

    async function handleAcknowledgement(ack) {
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

        let channelId
        try {
            channelId = await node.db.get(node.paymentChannels.ChannelId(ack.challengeSignatureHash))
        } catch (err) {
            if (err.notFound) {
                return
            }
            throw err
        }

        const ownKeyHalf = await node.db.get(node.paymentChannels.Challenge(channelId, secp256k1.publicKeyCreate(ack.key)))
        const channelKey = await node.db.get(node.paymentChannels.ChannelKey(channelId))

        node.db.batch()
            .put(node.paymentChannels.CHannelKey(channelId), secp256k1.privateKeyTweakAdd(channelKey, secp256k1.privateKeyTweakAdd(ack.key, ownKeyHalf)))
            .del(secp256k1.privateKeyTweakAdd(oldKey, key))
            .write()
    }

    node.handle(PROTOCOL_STRING, (protocol, conn) =>
        pull(
            conn,
            lp.decode({
                maxLength: Packet.SIZE
            }),
            paramap(async (data, cb) => {
                let packet
                try {
                    packet = await Packet.fromBuffer(data).forwardTransform(node)
                } catch (err) {
                    console.log(err)
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
