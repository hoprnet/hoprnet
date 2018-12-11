'use strict'

const Transaction = require('../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')
const pull = require('pull-stream')
const secp256k1 = require('secp256k1')
const { deepCopy, getId, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer } = require('../utils')
const { series } = require('async')

const SIGNATURE_LENGTH = 64

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    pull.filter((data) => data.length === Transaction.SIZE),
    pull.map((data) => Transaction.fromBuffer(data)),
    pull.filter(tx => getId(
        pubKeyToEthereumAddress(
            secp256k1.recover(tx.hash(), tx.signature, bufferToNumber(tx.recovery))),
        pubKeyToEthereumAddress(
            node.peerInfo.id.pubKey.marshal()
        )).compare(tx.channelId) === 0
    ),
    (read) => {
        let sigOpen, done = false

        return (end, reply) => {
            if (end) {
                reply(end, null)
            } else if (!sigOpen && !done) {
                series({
                    restoreTx: (cb) => read(end, cb),
                    tx: (cb) => read(end, cb)
                }, (err, { restoreTx, tx }) => {
                    if (err) { throw err }

                    // TODO: do some additional plausibility checks
                    if (
                        restoreTx.index !== 2 ** 32 - 1 ||
                        tx.index !== 0 ||
                        tx.value !== restoreTx.value ||
                        tx.channelId.compare(restoreTx.channelId) !== 0 ||
                        node.paymentChannels.has(tx.channelId)
                    ) {
                        reply(true, null)
                    } else {
                        node.paymentChannels.set(tx)
                        node.paymentChannels.setRestoreTransaction(restoreTx)
                        node.paymentChannels.registerSettlementListener(restoreTx.channelId)

                        const sigRestore = secp256k1.sign(restoreTx.hash(), node.peerInfo.id.privKey.marshal())
                        sigOpen = secp256k1.sign(tx.hash(), node.peerInfo.id.privKey.marshal())

                        reply(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
                    }
                })
            } else if (sigOpen && !done) {
                done = true
                reply(end, Buffer.concat([sigOpen.signature, numberToBuffer(sigOpen.recovery, 1)], SIGNATURE_LENGTH + 1))
            } else {
                reply(true, null)
            }
        }
    },
    conn
))