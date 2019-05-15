'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { log, bufferToNumber } = require('../../utils')
const Transaction = require('../../transaction')

module.exports = (self) => async (channelId, useRestoreTx = false) => {
    let lastTx, channelKey

    try {
        lastTx = Transaction.fromBuffer(await self.node.db.get(self.Transaction(channelId))
            .catch((err) => {
                if (!err.notFound)
                    throw err

                return self.node.db.get(self.RestoreTransaction(channelId))
            }))
    } catch (err) {
        console.log(chalk.red(err.message))
        return
    }
    
    log(self.node.peerInfo.id, `Trying to close payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m. Nonce is ${self.nonce}`)

    try {
        const receipt = await self.contractCall(self.contract.methods.closeChannel(
            lastTx.index,
            lastTx.nonce,
            (new BN(lastTx.value)).toString(),
            lastTx.curvePoint.slice(0, 32),
            lastTx.curvePoint.slice(32, 33),
            lastTx.signature.slice(0, 32),
            lastTx.signature.slice(32, 64),
            bufferToNumber(lastTx.recovery) + 27
        ))

        self.closingRequests.add(channelId.toString('base64'))
        log(self.node.peerInfo.id, `Settled channel \x1b[33m${channelId.toString('hex')}\x1b[0m with txHash \x1b[32m${receipt.transactionHash}\x1b[0m. Nonce is now \x1b[31m${self.nonce}\x1b[0m`)
    } catch (err) {
        console.log(err)
        return
    }
}