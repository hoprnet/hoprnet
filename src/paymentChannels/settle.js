'use strict'

const secp256k1 = require('secp256k1')

const { isPartyA, pubKeyToEthereumAddress, bufferToNumber } = require('../utils')

module.exports = (self) => (channelId, cb) => {
    const lastTx = self.get(channelId)

    const counterParty = pubKeyToEthereumAddress(
        secp256k1.recover(lastTx.hash(), lastTx.signature, bufferToNumber(lastTx.recovery))
    )

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
    }).then((receipt) => {
        const initialTx = self.getRestoreTransaction(channelId)

        let receivedMoney
        if (isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), counterParty)) {
            receivedMoney = lastTx.value - initialTx.value
        } else {
            receivedMoney = initialTx.value - lastTx.value
        }
        console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Finally receiving \'' + receivedMoney + '\' wei for channelId \'' + channelId.toString('base64') + '\'.')

        cb()
    })
}