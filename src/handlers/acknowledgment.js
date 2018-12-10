'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')

const { PROTOCOL_ACKNOWLEDGEMENT } = require('../constants')
const Acknowledgement = require('../acknowledgement')
const KeyDerivation = require('../paymentChannels/keyDerivation')

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

            const tx = node.pendingTransactions
                .get(ack.hashedKey.toString('base64'))
                .decrypt(ack.key)

            if (!tx.verify)
                throw Error('General error')
                
            console.log('Acknowledgement ' + (valid ? 'valid' : 'NOT VALID') + '.')
        }
    ]))
))