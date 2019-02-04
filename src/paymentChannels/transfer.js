'use strict'

const { waterfall } = require('neo-async')
const BN = require('bn.js')

const { isPartyA, getId, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer, deepCopy } = require('../utils')

const Transaction = require('../transaction')

module.exports = (self) => (amount, to, cb) => {
    let channelId
    waterfall([
        (cb) => {
            channelId = getId(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(to.pubKey.marshal()))

            self.getChannel(channelId, cb)
        },
        (record, cb) => {
            if (typeof record === 'function') {
                cb = record
                record = null
            }

            if (record) {
                cb(null, record)
            } else {
                self.open(to, cb)
            }
        },
        (record, cb) => {
            const currentValue = new BN(record.currentValue)

            const partyA = isPartyA(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(to.pubKey.marshal()))

            if (partyA) {
                currentValue.isub(amount)
                if (currentValue.isNeg())
                    cb(Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.abs().toString()} additional wei`))
            } else {
                currentValue.iadd(amount)

                const totalBalance = new BN(record.totalBalance)
                if (currentValue.gt(totalBalance))
                    cb(Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.sub(totalBalance).toString()} additional wei.`))
            }

            const newTx = deepCopy(record.tx, Transaction)

            newTx.value = currentValue.toBuffer('be', Transaction.VALUE_LENGTH)

            newTx.index = numberToBuffer(bufferToNumber(record.index) + 1, Transaction.INDEX_LENGTH)
            newTx.sign(self.node.peerInfo.id)

            self.setChannel({
                index: newTx.index,
                currentValue: newTx.value
            }, channelId, (err) => {
                if (err)
                    throw err

                cb(null, newTx)
            })
        }
    ], cb)
}