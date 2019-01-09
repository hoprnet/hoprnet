'use strict'

const secp256k1 = require('secp256k1')
const pull = require('pull-stream')
const { waterfall, map } = require('async')
const { bufferToNumber, pubKeyToPeerId } = require('../utils')

const bs58 = require('bs58')
const c = require('../constants')

const SETTLEMENT_TIMEOUT = 40000

module.exports = (self) => (cb) => {
    console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Closing channels ' + Array.from(self.openPaymentChannels.keys()).map(ch => `\x1b[33m${Buffer.from(ch, 'base64').toString('hex')}\x1b[0m`).join(', ') + '.')
    map(self.openPaymentChannels.values(), (record, cb) => {
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

                    // TODO: Implement proper transaction handling
                    const timeout = setTimeout(self.settle, SETTLEMENT_TIMEOUT, record.tx.channelId, true)

                    self.contract.once('ClosedChannel', {
                        topics: [`0x${record.restoreTx.channelId.toString('hex')}`]
                    }, (err) => {
                        if (err)
                            throw err

                        if (Date.now() - now < SETTLEMENT_TIMEOUT) {
                            // Prevent node from settling channel itself with a probably
                            // outdated transaction
                            clearTimeout(timeout)
                        }
                    })

                    pull(
                        pull.once(record.tx.channelId),
                        conn
                    )

                }),
            ], cb)
        } else {
            self.settle(record.tx.channelId)
        }

        self.on('closed ' + record.restoreTx.channelId.toString('base64'), (receivedMoney) => {
            // Callback just when the channel is settled, i.e. the closing listener
            // emits the 'closed <channelId>' event.
            cb(null, receivedMoney)
        })

    }, (err, results) => {
        if (err) { throw err }

        cb(null, results.reduce((acc, receivedMoney) => acc + receivedMoney, 0))
    })
}