'use strict'

const { isPartyA, pubKeyToEthereumAddress, log } = require('../utils')

module.exports = (self) => (channelId, useRestoreTx = false, cb = () => { }) => {
    if (typeof useRestoreTx === 'function') {
        cb = useRestoreTx
        useRestoreTx = false
    }

    let lastTx
    if (useRestoreTx) {
        lastTx = self.getRestoreTransaction(channelId)
    } else {
        lastTx = self.get(channelId)
    }

    if (lastTx) {
        log(self.node.peerInfo.id, `Trying to close payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m. Nonce is ${self.nonce}`)

        const counterParty = pubKeyToEthereumAddress(self.getCounterParty(channelId))

        const initialTx = self.getRestoreTransaction(channelId)

        self.contractCall(self.contract.methods.closeChannel(
            counterParty,
            lastTx.index,
            lastTx.value,
            lastTx.signature.slice(0, 32),
            lastTx.signature.slice(32, 64),
            lastTx.recovery
        ), (err, receipt) => {
            if (err) { throw err }

            let receivedMoney
            if (isPartyA(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), counterParty)) {
                receivedMoney = lastTx.value - initialTx.value
            } else {
                receivedMoney = initialTx.value - lastTx.value
            }

            log(self.node.peerInfo.id, `Settled channel \x1b[33m${channelId.toString('hex')}\x1b[0m with txHash \x1b[32m${receipt.transactionHash}\x1b[0m. Nonce is now \x1b[31m${self.nonce}\x1b[0m`)

            cb(null, receivedMoney)
        })
    } else {
        cb(null, 0)
    }

}