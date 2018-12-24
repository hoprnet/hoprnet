'use strict'

const { isPartyA, pubKeyToEthereumAddress, contractCall } = require('../utils')
const { DEFAULT_GAS_AMOUNT, GAS_PRICE } = require('../constants')

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
        console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Trying to close payment channel \'' + channelId.toString('hex') + '\'.')

        const counterParty = pubKeyToEthereumAddress(self.getCounterParty(channelId))

        const initialTx = self.getRestoreTransaction(channelId)

        // TODO this might fail when settling more than one transaction at the same time
        self.nonce = self.nonce + 1

        contractCall({
            nonce: self.nonce,
            to: self.contract._address,
            gas: 1000000,
            gasPrice: GAS_PRICE,
            data: self.contract.methods.settle(
                counterParty,
                lastTx.index,
                lastTx.value,
                lastTx.signature.slice(0, 32),
                lastTx.signature.slice(32, 64),
                lastTx.recovery
            ).encodeABI()
        }, self.node.peerInfo.id, self.node.web3, (err, hash) => {
            if (err) { throw err }

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
        cb(null, 0)
    }

}