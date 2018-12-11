'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')

const { PROTOCOL_ACKNOWLEDGEMENT } = require('../constants')
const { bufferXOR, hash } = require('../utils')
const Acknowledgement = require('../acknowledgement')

module.exports = (node) => node.handle(PROTOCOL_ACKNOWLEDGEMENT, (protocol, conn) => pull(
    conn,
    pull.filter(data =>
        data.length > 0 && data.length === Acknowledgement.SIZE
    ),
    pull.map(data => Acknowledgement.fromBuffer(data)),
    pull.drain(ack => waterfall([
        (cb) => conn.getPeerInfo(cb),
        (peerInfo, cb) => node.getPubKey(peerInfo, cb),
        (peerInfo, cb) => ack.verify(peerInfo.id.pubKey.marshal(), node.peerInfo.id.pubKey.marshal(), cb),
        (valid, cb) => {
            if (!node.pendingTransactions.has(ack.hashedKey.toString('base64')))
                throw Error('General error.')

            const { transaction, ownKeyHalf } = node.pendingTransactions
                .get(ack.hashedKey.toString('base64'))

            if (transaction && ownKeyHalf) {
                console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Decrypting with  \'' + hash(bufferXOR(ownKeyHalf, ack.key)).toString('base64') + '\'.')

                transaction.decrypt(hash(bufferXOR(ownKeyHalf, ack.key)))

                if (!transaction.verify(node))
                    throw Error('General error')

                node.paymentChannels.set(transaction)
                console.log('Acknowledgement ' + (valid ? 'valid' : 'NOT VALID') + '.')
            }
        }
    ]))
))