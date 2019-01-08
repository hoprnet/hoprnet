'use strict'

const { waterfall } = require('async')
const { isPartyA, pubKeyToEthereumAddress, mineBlock, log } = require('../utils')

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
            (channel, cb) => self.node.web3.eth.getBlockNumber((err, blockNumber) => cb(err, blockNumber, channel)),
            (blockNumber, channel, cb) => {
                if (blockNumber < channel.settlementBlock) {
                    const subscription = self.node.web3.eth.subscribe('newBlockHeaders')
                        .on('data', (block) => {
                            console.log('Waiting ... Block \'' + block.number + '\'.')
                            if (block.number > parseInt(channel.settlementBlock)) {
                                subscription.unsubscribe((err, ok) => {
                                    if (ok)
                                        cb(err)
                                })
                            }
                            else {
                                // ================ Only for testing ================
                                 mineBlock(self.contract.currentProvider)
                                // ==================================================

                            }
                        })

                    // ================ Only for testing ================
                     mineBlock(self.contract.currentProvider)
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

                    self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(counterParty)), cb)
                } else {
                    cb()
                }
            }
        ], (err, receipt) => {
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

            log(self.node.peerInfo.id, `Closed payment channel \x1b[33m${initialTx.channelId.toString('hex')}\x1b[0m and ${receivedMoney < 0 ? 'spent' : 'received'} \x1b[35m${Math.abs(receivedMoney)} wei\x1b[0m. ${receipt ? ` TxHash \x1b[32m${receipt.transactionHash}.` : ''}`)

            self.delete(lastTx.channelId)

            if (interested) {
                self.emit('closed ' + channelId.toString('base64'), receivedMoney)
            }
        })
    }
} 