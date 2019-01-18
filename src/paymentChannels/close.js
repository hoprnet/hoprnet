'use strict'

const { waterfall } = require('neo-async')
const { isPartyA, pubKeyToEthereumAddress, mineBlock, log } = require('../utils')
const { NET } = require('../constants')
const { BN } = require('web3-utils')

module.exports = (self) => (err, event) => {
    if (err)
        throw err

    const channelId = Buffer.from(event.returnValues.channelId.slice(2), 'hex')

    self.getChannel(channelId, (err, record) => {
        if (err)
            throw err

        if (!record)
            log(self.node.peerInfo.id, `Listening to wrong channel. ${channelId.toString('hex')}.`)

        const { tx, restoreTx, index } = record
        const amountA = new BN(event.returnValues.amountA)
        const counterparty = record.restoreTx.counterparty

        let interested = false

        const partyA = isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(counterparty)
        )

        waterfall([
            (cb) => {
                if (
                    Buffer.from(event.returnValues.index.replace('0x', ''), 'hex').compare(tx.index) === -1 &&
                    (partyA ? new BN(tx.value).gt(amountA) : amountA.gt(new BN(tx.value)))
                ) {
                    log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

                    self.settle(channelId, cb)
                } else {
                    cb(null)
                }
            },
            (cb) => self.contract.methods.channels(channelId).call({
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            }, cb),
            (channel, cb) => self.node.web3.eth.getBlock((err, block) => cb(err, block, channel)),
            (block, channel, cb) => {
                if (block.timestamp < parseInt(channel.settleTimestamp)) {
                    const subscription = self.node.web3.eth.subscribe('newBlockHeaders')
                        .on('data', (block) => {
                            log(self.node.peerInfo.id, `Waiting ... Block ${block.number}.`)

                            if (block.timestamp > parseInt(channel.settleTimestamp)) {
                                subscription.unsubscribe((err, ok) => {
                                    if (ok)
                                        cb(err)
                                })
                            } else if (NET === 'ganache') {
                                // ================ Only for testing ================
                                mineBlock(self.contract.currentProvider)
                                // ==================================================

                            }
                        })

                    if (NET === 'ganache') {
                        // ================ Only for testing ================
                        mineBlock(self.contract.currentProvider)
                        // ==================================================
                    }

                } else {
                    cb()
                }
            },
            (cb) => {
                if (self.eventNames().some((name) =>
                    name === `closed ${channelId.toString('base64')}`
                )) {
                    interested = true

                    self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(counterparty)), cb)
                } else {
                    cb()
                }
            }
        ], (err, receipt) => {
            if (err)
                throw err

            const initialValue = new BN(restoreTx.value)

            const receivedMoney = partyA ? amountA.isub(initialValue) : initialValue.isub(amountA)

            log(self.node.peerInfo.id, `Closed payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m and ${receivedMoney.isNeg() ? 'spent' : 'received'} \x1b[35m${receivedMoney.abs().toString()} wei\x1b[0m. ${receipt ? ` TxHash \x1b[32m${receipt.transactionHash}\x1b[0m.` : ''}`)

            self.deleteChannel(channelId, (err) => {
                if (err)
                    throw err

                if (interested) {
                    self.emit(`closed ${channelId.toString('base64')}`, receivedMoney)
                }
            })
        })
    })
} 