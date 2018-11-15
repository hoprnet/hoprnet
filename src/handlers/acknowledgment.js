'use strict'

const pull = require('pull-stream')
const waterfall = require('async/waterfall')

const { PROTOCOL_ACKNOWLEDGEMENT, PROTOCOL_DELIVER_PUBKEY, COM} = require('../constants')
const Acknowledgement = require('../acknowledgement')
const KeyDerivation = require('../../old/payments/keyDerivation')

module.exports = (node) => node.handle(PROTOCOL_ACKNOWLEDGEMENT, (protocol, conn) => {
    pull(
        conn,
        pull.filter(data =>
            data.length > 0 && data.length === Acknowledgement.SIZE
        ),
        pull.map(data => Acknowledgement.fromBuffer(data)),
        pull.drain(ack => {
            waterfall([
                (cb) => conn.getPeerInfo(cb),
                (peerInfo, cb) => node.getPubKey(peerInfo, cb),
                (pubKey, cb) => {
                    ack.verify(pubKey, node.peerInfo.id.pubKey.marshal(), cb)
                },
                (valid, cb) => {
                    console.log('Acknowledgement ' + (valid ? 'valid' : 'NOT VALID') + '.')
                }
            ], (err) => {
                // console.log(err)
            })
        })
    )
})