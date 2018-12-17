'use strict'

const secp256k1 = require('secp256k1')

const { isPartyA, pubKeyToEthereumAddress, bufferToNumber } = require('../utils')

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
            gas: 250333, // arbitrary
            gasPrice: '30000000000000'
        }).on('error', (err) => {
            console.log(err)
        })
        
        .then((receipt) => {    
            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Settled payment channel \'' + channelId.toString('hex') + '\'.')

            let receivedMoney
            if (isPartyA(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), counterParty)) {
                receivedMoney = lastTx.value - initialTx.value
            } else {
                receivedMoney = initialTx.value - lastTx.value
            }
            // console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Finally receiving \'' + receivedMoney + '\' wei in channel \'' + channelId.toString('base64') + '\'.')
    
            cb()
        })
    } else {
        cb()
    }

}