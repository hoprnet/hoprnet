'use strict'

const { log, bufferToNumber } = require('../../utils')
const { waterfall } = require('neo-async')
const BN = require('bn.js')

module.exports = (self) => (channelId, useRestoreTx = false) => {
    let lastTx

    waterfall([
        (cb) => self.getChannel(channelId, cb),
        (record, cb) => {
            if (typeof record === 'function') {
                cb = record

                return cb(null, new BN('0'))
            }

            lastTx = useRestoreTx ? record.restoreTx : record.tx

            log(self.node.peerInfo.id, `Trying to close payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m. Nonce is ${self.nonce}`)

            self.getChannelKey(channelId, cb)
        },
        (channelKey, cb) =>
            self.contractCall(self.contract.methods.closeChannel(
                lastTx.index,
                lastTx.nonce,
                (new BN(lastTx.value)).toString(),
                lastTx.curvePoint.slice(0, 32),
                lastTx.curvePoint.slice(32, 33),
                lastTx.signature.slice(0, 32),
                lastTx.signature.slice(32, 64),
                bufferToNumber(lastTx.recovery) + 27
            ), cb)
    ], (err, receipt) => {
        if (err) {
            console.log(err)
            return
        }

        self.closingRequests.add(channelId.toString('base64'))
        log(self.node.peerInfo.id, `Settled channel \x1b[33m${channelId.toString('hex')}\x1b[0m with txHash \x1b[32m${receipt.transactionHash}\x1b[0m. Nonce is now \x1b[31m${self.nonce}\x1b[0m`)
    })
}