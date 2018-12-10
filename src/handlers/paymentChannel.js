'use strict'

const Transaction = require('../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')
const pull = require('pull-stream')
const secp256k1 = require('secp256k1')
const { getId, pubKeyToEthereumAddress, bufferToNumber } = require('../utils')
const { waterfall } = require('async')

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    pull.filter((data) => data.length === Transaction.SIZE),
    pull.map((data) => Transaction.fromBuffer(data)),
    (read) => (end, reply) => waterfall([
        (cb) => read(end, cb),
        (tx, cb) => {
            const pubKey = secp256k1.recover(tx.hash(), tx.signature, bufferToNumber(tx.recovery))

            const channelId = getId(
                pubKeyToEthereumAddress(pubKey),
                pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()))

            if (channelId.compare(tx.channelId) !== 0)
                throw Error('General error.')

            node.paymentChannels.set(tx)
            node.paymentChannels.registerSettlementListener(tx.channelId)

            cb(null, secp256k1.sign(
                tx.hash(),
                node.peerInfo.id.privKey.marshal()).signature)
        }
    ], reply),
    conn
))