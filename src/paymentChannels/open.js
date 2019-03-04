'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const { waterfall } = require('neo-async')
const { randomBytes } = require('crypto')
const { toWei } = require('web3-utils')
const BN = require('bn.js')
const { deepCopy, bufferToNumber, numberToBuffer, log } = require('../utils')
const { recover } = require('secp256k1')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')

const Transaction = require('../transaction')

module.exports = (self) => (to, cb) => {
    let restoreTx

    waterfall([
        (cb) => self.node.peerRouting.findPeer(to, cb),
        (peerInfo, cb) => self.node.dialProtocol(peerInfo, PROTOCOL_PAYMENT_CHANNEL, cb),
        (conn, cb) => {
            restoreTx = new Transaction()

            restoreTx.nonce = randomBytes(Transaction.NONCE_LENGTH)
            restoreTx.value = (new BN(toWei('1', 'shannon'))).toBuffer('be', Transaction.VALUE_LENGTH)
            restoreTx.index = numberToBuffer(1, Transaction.INDEX_LENGTH)

            restoreTx.sign(self.node.peerInfo.id)

            pull(
                pull.once(restoreTx.toBuffer()),
                lp.encode(),
                conn,
                lp.decode(),
                pull.filter((data) =>
                    Buffer.isBuffer(data) &&
                    data.length === Transaction.SIGNATURE_LENGTH + Transaction.RECOVERY_LENGTH &&
                    recover(restoreTx.hash, data.slice(0, Transaction.SIGNATURE_LENGTH), bufferToNumber(data.slice(Transaction.SIGNATURE_LENGTH)))
                        .compare(to.pubKey.marshal()) === 0
                ),
                pull.collect(cb)
            )
        },
        (signatures, cb) => {
            if (signatures.length !== 1)
                return cb(Error(`Invalid response. To: ${to.toB58String()}`))

            restoreTx.signature = signatures[0].slice(0, Transaction.SIGNATURE_LENGTH)
            restoreTx.recovery = signatures[0].slice(Transaction.SIGNATURE_LENGTH)

            console.log(restoreTx.getChannelId(self.node.peerInfo.id).toString('hex'))
            return self.contractCall(self.contract.methods.createFunded(
                restoreTx.nonce,
                (new BN(restoreTx.value)).toString(),
                restoreTx.signature.slice(0, 32),
                restoreTx.signature.slice(32, 64),
                bufferToNumber(restoreTx.recovery) + 27
            ), cb)
        },
        (receipt, cb) => {
            self.setSettlementListener(restoreTx.getChannelId(self.node.peerInfo.id))

            const newRecord = {
                restoreTx: restoreTx,
                tx: deepCopy(restoreTx, Transaction),
                index: restoreTx.index,
                currentValue: restoreTx.value,
                totalBalance: (new BN(restoreTx.value)).imuln(2).toBuffer('be', Transaction.VALUE_LENGTH)
            }

            return self.setChannel(newRecord, { sync: true }, (err) => {
                if (err)
                    return cb(err)

                log(self.node.peerInfo.id, `Opened payment channel \x1b[33m${restoreTx.getChannelId(self.node.peerInfo.id).toString('hex')}\x1b[0m with txHash \x1b[32m${receipt.transactionHash}\x1b[0m. Nonce is now \x1b[31m${self.nonce - 1}\x1b[0m.`)

                return cb(null, newRecord)
            })
        }
    ], cb)
}