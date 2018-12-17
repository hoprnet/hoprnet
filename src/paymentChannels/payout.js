'use strict'

const secp256k1 = require('secp256k1')
const pull = require('pull-stream')
const { waterfall, each } = require('async')
const { bufferToNumber, pubKeyToPeerId } = require('../utils')

const bs58 = require('bs58')
const c = require('../constants')

const SETTLEMENT_TIMEOUT = 3000

module.exports = (self) => (cb) => {
    console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Closing channels ' + Array.from(self.openPaymentChannels.keys()).join(', ') + '.')
    each(self.openPaymentChannels.values(), (record, cb) => {
        let counterParty = secp256k1.recover(record.tx.hash, record.tx.signature, bufferToNumber(record.tx.recovery))

        if (self.node.peerInfo.id.pubKey.marshal().compare(counterParty) === 0) {
            waterfall([
                (cb) => pubKeyToPeerId(
                    secp256k1.recover(
                        record.restoreTx.hash,
                        record.restoreTx.signature,
                        bufferToNumber(record.restoreTx.recovery)), cb),
                (peerId, cb) => self.node.peerRouting.findPeer(peerId, cb),
                (peerInfo, cb) => self.node.dialProtocol(peerInfo, c.PROTOCOL_SETTLE_CHANNEL, (err, conn) => {
                    if (err) { throw err }

                    const now = Date.now()

                    const timeout = setTimeout(self.settle, SETTLEMENT_TIMEOUT, record.tx.channelId, true, cb)

                    const listener = record.listener
                    self.setSettlementListener(record.tx.channelId, (err, event, subscription) => {
                        console.log('Decorated listener.')
                        if (Date.now() - now < SETTLEMENT_TIMEOUT) {
                            clearTimeout(timeout)
                            console.log('Timeout cleared.')
                            // self.close(err, event, null, cb)
                        }
                        listener(err, event, subscription, cb)
                    })

                    pull(
                        pull.once(record.tx.channelId),
                        conn
                    )

                }),
            ], cb)
        } else {
            self.settle(record.tx.channelId, cb)
        }
    }, (err) => {
        throw Error('foo')
    })
}