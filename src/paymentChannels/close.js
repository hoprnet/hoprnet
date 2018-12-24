'use strict'

const { waterfall } = require('async')
const { isPartyA, pubKeyToEthereumAddress, mineBlock, contractCall } = require('../utils')
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
            (channel, cb) => self.node.web3.eth.getBlockNumber((err, blockNumber) => cb(err, blockNumber, channel)),
            (blockNumber, channel, cb) => {
                if (blockNumber < channel.settlementBlock) {
                    const subscription = self.node.web3.eth.subscribe('newBlockHeaders')
                        .on('data', (block) => {
                            console.log('Waiting ... Block \'' + block.number + '\'.')
                            if (block.number > parseInt(channel.settlementBlock)) {
                                subscription.unsubscribe((err, ok) => cb(err))
                            }
                            else {
                                // ================ Only for testing ================
                                //mineBlock(self.contract.currentProvider)
                                // ==================================================

                            }
                        })

                    // ================ Only for testing ================
                    //mineBlock(self.contract.currentProvider)
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

                    contractCall({
                        to: self.contract._address,
                        gas: 1000000,
                        gasPrice: GAS_PRICE,
                        data: self.contract.methods.withdraw(pubKeyToEthereumAddress(counterParty)).encodeABI()
                    }, self.node.peerInfo.id, self.node.web3, cb)
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

            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Closed payment channel \'' + channelId.toString('hex') + '\' and received ' + receivedMoney + ' wei.' + (hash ? ' TxHash \''.concat(hash).concat('\'.') : ''))

            self.delete(lastTx.channelId)

            if (interested) {
                self.emit('closed ' + channelId.toString('base64'), receivedMoney)
            }
        })
    }
} 