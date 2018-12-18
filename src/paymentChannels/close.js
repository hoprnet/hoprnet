'use strict'

const { waterfall } = require('async')
const { isPartyA, pubKeyToEthereumAddress, mineBlocks } = require('../utils')

const CONFIRMATION_TIME = 15

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

    return (err, event) => {
        if (err) { throw err }

        const channelId = Buffer.from(event.returnValues.channelId.slice(2), 'hex')

        if (!self.has(channelId)) {
            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Listening to wrong channel. Channel \'' + channelId.toString('hex') + '\'.')
            return
        }



        const amountA = parseInt(event.returnValues.amountA)
        const lastTx = self.get(channelId)
        const counterParty = self.getCounterParty(channelId)

        let interested = false

        waterfall([
            (cb) => {
                if (
                    parseInt(event.returnValues.index) < parseInt(lastTx.index) &&
                    hasBetterTx(channelId, amountA, counterParty)
                ) {
                    console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Found better transaction for payment channel \'' + channelId.toString('hex') + '\'.')

                    self.settle(lastTx.channelId, cb)
                } else {
                    cb(null)
                }
            },
            (cb) => {
                const subscription = self.node.eth.subscribe('newBlockHeaders')
                    .on('data', (block) => {
                        if (block.number > event.blockNumber + CONFIRMATION_TIME) {
                            subscription.unsubscribe(cb)
                        }
                    })

                // Only for testing!
                mineBlocks(self.contract.currentProvider, CONFIRMATION_TIME + 3)
            },
            (_, cb) => {
                if (self.eventNames().some((name) => 
                    name === 'closed '.concat(channelId.toString('base64'))
                )) {
                    interested = true
                    self.contract.methods.withdraw(pubKeyToEthereumAddress(counterParty)).send({
                        from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                        gas: 250333, // arbitrary
                        gasPrice: '30000000000000'
                    }, cb)
                } else {
                    cb()
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

            self.delete(lastTx.channelId)

            if (interested) {
                self.emit('closed ' + channelId.toString('base64'), receivedMoney)
            }
        })
    }
} 