'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const { waterfall } = require('neo-async')
const { randomBytes } = require('crypto')
const { toWei } = require('web3-utils')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { deepCopy, bufferToNumber, numberToBuffer } = require('../../utils')
const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const Transaction = require('../../transaction')
const Record = require('../record')

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

            // 0 is considered as infinity point / neutral element
            restoreTx.curvePoint = Buffer.alloc(33, 0)

            restoreTx.sign(self.node.peerInfo.id)

            pull(
                pull.once(restoreTx.toBuffer()),
                lp.encode(),
                conn,
                lp.decode(),
                pull.filter((data) =>
                    Buffer.isBuffer(data) &&
                    data.length === Transaction.SIGNATURE_LENGTH + Transaction.RECOVERY_LENGTH &&
                    secp256k1.recover(restoreTx.hash, data.slice(0, Transaction.SIGNATURE_LENGTH), bufferToNumber(data.slice(Transaction.SIGNATURE_LENGTH)))
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

            const channelId = restoreTx.getChannelId(self.node.peerInfo.id)

            self.openingRequests.set(channelId.toString('base64'), Record.create(
                restoreTx,
                deepCopy(restoreTx, Transaction),
                restoreTx.index,
                restoreTx.value,
                (new BN(restoreTx.value)).imuln(2).toBuffer('be', Transaction.VALUE_LENGTH)
            ))

            self.registerSettlementListener(channelId)
            self.registerOpeningListener(channelId)

            self.once(`opened ${channelId.toString('base64')}`, cb)

            self.contractCall(self.contract.methods.createFunded(
                restoreTx.nonce,
                (new BN(restoreTx.value)).toString(),
                restoreTx.signature.slice(0, 32),
                restoreTx.signature.slice(32, 64),
                bufferToNumber(restoreTx.recovery) + 27
            ))
        },
    ], cb)
}