'use strict'

const Transaction = require('../packet/transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')
const pull = require('pull-stream')
const secp256k1 = require('secp256k1')
const { getId, pubKeyToEthereumAddress, bufferToNumber } = require('../utils')
const { waterfall } = require('async')

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    pull.filter((data) => data.length === Transaction.SIZE),
    pull.map((data) => {
        return Transaction.fromBuffer(data)
    }),
    (read) => (end, reply) => waterfall([
        (cb) => read(end, cb),
        (transaction, cb) => {
            const pubKey = secp256k1.recover(transaction.hash(), transaction.signature, bufferToNumber(transaction.recovery))

            if (getId(
                pubKeyToEthereumAddress(pubKey),
                pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
            ).compare(transaction.channelId) !== 0)
                throw Error('General error.')

            node.openPaymentChannels.set(transaction.channelId.toString('base64'), transaction)

            cb(null, secp256k1.sign(
                transaction.hash(),
                node.peerInfo.id.privKey.marshal()).signature)
        }
    ], (err, data) => {
        reply(err, data)
    }),
    conn
))