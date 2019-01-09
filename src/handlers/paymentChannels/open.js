'use strict'

const Transaction = require('../../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const pull = require('pull-stream')
const secp256k1 = require('secp256k1')
const { deepCopy, getId, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer } = require('../../utils')
const { waterfall } = require('neo-async')

const SIGNATURE_LENGTH = 64

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    pull.filter((data) => data.length === Transaction.SIZE),
    pull.map((data) => Transaction.fromBuffer(data)),
    pull.filter(restoreTx =>
        restoreTx.index === 1 &&
        getId(
            pubKeyToEthereumAddress(
                secp256k1.recover(restoreTx.hash, restoreTx.signature, bufferToNumber(restoreTx.recovery))),
            pubKeyToEthereumAddress(
                node.peerInfo.id.pubKey.marshal()
            )).compare(restoreTx.channelId) === 0
    ),
    (read) => (end, reply) => waterfall([
        (cb) => {
            if (end) {
                cb(end, null)
            } else {
                read(end, cb)
            }
        },
        (restoreTx, cb) => {
            node.paymentChannels.setRestoreTransaction(restoreTx)
            node.paymentChannels.set(deepCopy(restoreTx, Transaction))

            node.paymentChannels.setSettlementListener(restoreTx.channelId)

            const sigRestore = secp256k1.sign(restoreTx.hash, node.peerInfo.id.privKey.marshal())
            cb(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
        }
    ], reply),
    conn
))