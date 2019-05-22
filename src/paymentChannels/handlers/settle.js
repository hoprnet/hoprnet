'use strict'

const chalk = require('chalk')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const { PROTOCOL_SETTLE_CHANNEL } = require('../../constants')
const Transaction = require('../../transaction')
const { log } = require('../../utils')

const CHANNEL_ID_LENGTH = 32

module.exports = (node) => node.handle(PROTOCOL_SETTLE_CHANNEL, (protocol, conn) => pull(
    conn,
    lp.decode({
        maxLength: CHANNEL_ID_LENGTH
    }),
    pull.asyncMap((channelId, cb) => {
        if (channelId.length != CHANNEL_ID_LENGTH)
            return cb(null, Buffer.alloc(0))

        node.paymentChannels.getLastTransaction(channelId)
            .catch((err) => {
                log(node.peerInfo.id, chalk.red(err.message))
                cb(null, Buffer.alloc(0))
            })
            .then((lastTx) => {
                const buf = Buffer.alloc(Transaction.SIGNATURE_LENGTH + Transaction.RECOVERY_LENGTH)
                lastTx.signature.copy(buf, 0, 0, Transaction.SIGNATURE_LENGTH)
                lastTx.recovery.copy(buf, Transaction.SIGNATURE_LENGTH, 0, Transaction.RECOVERY_LENGTH)

                return cb(null, Buffer.concat([lastTx.sign(node.peerInfo.id).toBuffer(), buf], Transaction.SIZE + Transaction.SIGNATURE_LENGTH + Transaction.RECOVERY_LENGTH))
            })
    }),
    lp.encode(),
    conn
))