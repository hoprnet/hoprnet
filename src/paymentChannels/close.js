'use strict'

const { waterfall } = require('async')
const { isPartyA, pubKeyToEthereumAddress, mineBlock } = require('../utils')
const { DEFAULT_GAS_AMOUNT, GAS_PRICE } = require('../constants')

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
            (cb) => self.contract.methods.channels(channelId).call({
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            }, cb),
            (channel, cb) => self.node.eth.getBlockNumber((err, blockNumber) => cb(err, blockNumber, channel)),
            (blockNumber, channel, cb) => {
                if (blockNumber < channel.settlementBlock) {
                    const subscription = self.node.eth.subscribe('newBlockHeaders')
                        .on('data', (block) => {
                            if (block.number > parseInt(channel.settlementBlock)) {
                                subscription.unsubscribe((err, ok) => cb(err))
                            }
                            // else {
                            //     // ================ Only for testing ================
                            //     mineBlock(self.contract.currentProvider)
                            //     // ==================================================

                            // }
                        })

                    // ================ Only for testing ================
                    // mineBlock(self.contract.currentProvider)
                    // ==================================================
                } else {
                    cb()
                }
            },
            (cb) => {
                if (self.eventNames().some((name) =>
                    name === 'closed '.concat(channelId.toString('base64'))
                )) {
                    interested = true
                    self.nonce = self.nonce + 1
                    self.contract.methods.withdraw(pubKeyToEthereumAddress(counterParty)).send({
                        from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                        gas: DEFAULT_GAS_AMOUNT, // arbitrary
                        gasPrice: GAS_PRICE
                    }, cb)
                } else {
                    cb()
                }
            }
        ], (err, hash) => {
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

            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Closed payment channel \'' + channelId.toString('hex') + '\' and received ' + receivedMoney + ' wei. TxHash \'' + hash + '\'.')

            self.delete(lastTx.channelId)

            if (interested) {
                self.emit('closed ' + channelId.toString('base64'), receivedMoney)
            }
        })
    }
} 