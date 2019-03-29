'use strict'

const { waterfall } = require('neo-async')
const { isPartyA, pubKeyToEthereumAddress, mineBlock, log } = require('../../utils')
const { NETWORK } = require('../../constants')
const BN = require('bn.js')

module.exports = (self) => (err, event) => {
    if (err)
        throw err

    const channelId = Buffer.from(event.raw.topics[1].slice(2), 'hex')
    const amountA = new BN(event.returnValues.amountA)

    let receivedMoney = new BN(0), record, counterparty, partyA

    waterfall([
        (cb) => self.getChannel(channelId, cb),
        (_record, cb) => {
            if (typeof _record === 'function') {
                cb = _record
                return cb(Error(`Listening to wrong channel. ${channelId.toString('hex')}.`))
            }

            record = _record
            counterparty = record.restoreTx.counterparty

            partyA = isPartyA(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(counterparty)
            )

            if (
                Buffer.from(event.returnValues.index.replace(/0x/, ''), 'hex').compare(record.tx.index) === -1 &&
                (partyA ? new BN(record.tx.value).gt(amountA) : amountA.gt(new BN(record.tx.value)))
            ) {
                log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

                self.registerSettlementListener(channelId)
                self.requestClose(channelId)
            } else {
                cb()
            }
        },
        (cb) => self.contract.methods.channels(channelId).call({
            from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
        }, 'latest', cb),
        (channel, cb) => {
            const subscription = self.web3.eth.subscribe('newBlockHeaders').on('data', (block) => {
                log(self.node.peerInfo.id, `Waiting ... Block ${block.number}.`)

                if (block.timestamp > parseInt(channel.settleTimestamp)) {
                    subscription.unsubscribe((err, ok) => {
                        if (err)
                            return cb(err)

                        if (ok)
                            cb()
                    })
                } else if (NETWORK === 'ganache') {
                    // ================ Only for testing ================
                    mineBlock(self.contract.currentProvider)
                    // ==================================================
                }
            })

            if (NETWORK === 'ganache') {
                // ================ Only for testing ================
                mineBlock(self.contract.currentProvider)
                // ==================================================
            }
        },
        (cb) => {
            if (!self.closingRequests.has(channelId.toString('base64')))
                return cb()

            self.closingRequests.delete(channelId.toString('base64'))
            self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(counterparty)), cb)
        },
        (receipt, cb) => {
            if (typeof receipt === 'function') {
                cb = receipt
                receipt = null
            }

            const initialValue = new BN(record.restoreTx.value)
            receivedMoney = partyA ? amountA.isub(initialValue) : initialValue.isub(amountA)

            log(self.node.peerInfo.id, `Closed payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m and ${receivedMoney.isNeg() ? 'spent' : 'received'} \x1b[35m${receivedMoney.abs().toString()} wei\x1b[0m. ${receipt ? ` TxHash \x1b[32m${receipt.transactionHash}\x1b[0m.` : ''}`)

            self.deleteChannel(channelId, cb)
        }
    ], (err) => {
        if (err)
            console.log(err)

        self.emit(`closed ${channelId.toString('base64')}`, receivedMoney)
    })
}