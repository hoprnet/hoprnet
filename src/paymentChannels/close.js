'use strict'

const { waterfall } = require('async')
const { isPartyA, pubKeyToEthereumAddress } = require('../utils')

module.exports = (self) => {
    function hasBetterTx(channelId, amountA, counterParty) {
        const lastTx = self.get(channelId)

        if (isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(counterParty)
        )) {
            return lastTx.value > amountA
        } else {
            return amountA > lastTx.value
        }
    }

    return (err, event, _, cb) => {
        if (err) { throw err }

        const channelId = Buffer.from(event.returnValues.channelId.slice(2), 'hex')

        if (!self.has(channelId)) {
            // console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Listening to wrong channel. Channel \'' + channelId.toString('hex') + '\'.')
            return
        }

        const amountA = parseInt(event.returnValues.amountA)
        const lastTx = self.get(channelId)
        const counterParty = self.getCounterParty(channelId)

        waterfall([
            (cb) => {
                if (
                    parseInt(event.returnValues.index) < parseInt(lastTx.index) &&
                    hasBetterTx(channelId, amountA, counterParty)
                ) {
                    console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Found better transaction for payment channel \'' + channelId.toString('hex') + '\' and received ' + receivedMoney + ' wei.')

                    self.settle(lastTx.channelId, cb)
                } else {
                    cb(null)
                }
            }
        ], (err) => {
            if (err) { throw err }

            let receivedMoney
            const initialTx = self.getRestoreTransaction(channelId)
            if (isPartyA(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(counterParty)
            )) {
                receivedMoney = amountA - initialTx.value
            } else {
                receivedMoney = initialTx.value - amountA
            }

            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Closed payment channel \'' + channelId.toString('hex') + '\' and received ' + receivedMoney + ' wei.')

            // self.delete(lastTx.channelId)

            if (cb)
                cb()
        })
    }
} 