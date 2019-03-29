'use strict'

const { waterfall, queue } = require('neo-async')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { isPartyA, getId, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer, deepCopy } = require('../utils')
const Transaction = require('../transaction')

module.exports = (self) => {
    const queues = new Map()

    const foo = (options, cb) => {
        let newTx, record

        waterfall([
            (cb) => self.getChannel(options.channelId, cb),
            (record, cb) => {
                if (typeof record === 'function') {
                    cb = record
                    record = null
                }

                if (record)
                    return cb(null, record)

                if (self.openingRequests.has(options.channelId.toString('base64')))
                    console.log('found')

                self.open(options.to, cb)

            },
            (_record, cb) => {
                record = _record
                const currentValue = new BN(record.currentValue)

                const partyA = isPartyA(
                    pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                    pubKeyToEthereumAddress(options.to.pubKey.marshal()))

                if (partyA) {
                    currentValue.isub(options.amount)
                    if (currentValue.isNeg())
                        return cb(Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.abs().toString()} additional wei.`))
                } else {
                    currentValue.iadd(options.amount)

                    const totalBalance = new BN(record.totalBalance)
                    if (currentValue.gt(totalBalance))
                        return cb(Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.sub(totalBalance).toString()} additional wei.`))
                }

                newTx = deepCopy(record.tx, Transaction)

                newTx.value = currentValue.toBuffer('be', Transaction.VALUE_LENGTH)
                newTx.index = numberToBuffer(bufferToNumber(record.index) + 1, Transaction.INDEX_LENGTH)

                record.index = numberToBuffer(bufferToNumber(record.index) + 1, Transaction.INDEX_LENGTH)
                record.currentValue = currentValue.toBuffer('be', Transaction.VALUE_LENGTH)

                self.getChannelKey(options.channelId, cb)
            },
            (key, cb) => {
                newTx.curvePoint = secp256k1.publicKeyCreate(key)

                newTx.sign(self.node.peerInfo.id)

                self.setChannel(record, { channelId: options.channelId, sync: true }, (err) => {
                    if (err)
                        return cb(err)

                    options.done(null, newTx)

                    setImmediate(cb)
                })
            }
        ], cb)
    }

    return (amount, to, cb) => {
        const channelId = getId(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal())
        )

        let pendingJobs = queues.get(channelId.toString('base64'))
        if (!pendingJobs) {
            pendingJobs = queue(foo, 1)
        }

        pendingJobs.push({
            amount: amount,
            to: to,
            channelId: channelId,
            done: cb
        })

        queues.set(channelId.toString('base64'), pendingJobs)
    }
}