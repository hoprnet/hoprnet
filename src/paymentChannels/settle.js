'use strict'

const { isPartyA, pubKeyToEthereumAddress } = require('../utils')
const { DEFAULT_GAS_AMOUNT, GAS_PRICE } = require('../constants')

module.exports = (self) => (channelId, useRestoreTx = false, cb = () => {}) => {
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
        console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Trying to close payment channel \'' + channelId.toString('hex') + '\'.')

        const counterParty = pubKeyToEthereumAddress(self.getCounterParty(channelId))
        
        const initialTx = self.getRestoreTransaction(channelId)
    
        self.contract.methods.settle(
            counterParty,
            lastTx.index,
            lastTx.value,
            lastTx.signature.slice(0, 32),
            lastTx.signature.slice(32, 64),
            lastTx.recovery
        ).send({
            from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            gas: DEFAULT_GAS_AMOUNT, // arbitrary
            gasPrice: GAS_PRICE
        }, (err, txHash) => {
            if (err) { throw err }

            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Settled payment channel \'' + channelId.toString('hex') + '\'.')

            let receivedMoney
            if (isPartyA(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), counterParty)) {
                receivedMoney = lastTx.value - initialTx.value
            } else {
                receivedMoney = initialTx.value - lastTx.value
            }
            
            cb(null, receivedMoney)

        })
    } else {
        cb()
    }

}