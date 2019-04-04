'use strict'

const Queue = require('promise-queue')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { isPartyA, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer } = require('../utils')
const Transaction = require('../transaction')

module.exports = (self) => {
    const queues = new Map()

    async function transfer(options) {
        let tx, channelKey

        try {
            tx = Transaction.fromBuffer(await self.node.db.get(self.RestoreTransaction(options.channelId)))
        } catch (err) {
            if (err.notFound) {
                try {
                    await self.node.db.get(self.StashedRestoreTransaction(options.channelId))
                    tx = await new Promise((resolve, reject) => self.once(`opened ${options.channelId.toString('base64')}`, resolve))
                } catch (err) {
                    if (err.notFound) {
                        tx = await self.open(options.to)
                    }
                }
            } else {
                throw err
            }
        }

        try {
            channelKey = secp256k1.privateKeyTweakAdd(await self.node.db.get(self.ChannelKey(options.channelId)), options.key)
        } catch (err) {
            if (err.notFound) {
                channelKey = options.key
            } else {
                throw err
            }
        }

        const currentValue = new BN(await self.node.db.get(self.CurrentValue(options.channelId)))

        const partyA = isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(options.to.pubKey.marshal()))

        if (partyA) {
            currentValue.isub(options.amount)
            if (currentValue.isNeg())
                throw Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.abs().toString()} additional wei.`)
        } else {
            currentValue.iadd(options.amount)

            const totalBalance = new BN(await self.node.db.get(self.TotalBalance(options.channelId)))
            if (currentValue.gt(totalBalance))
                throw Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.sub(totalBalance).toString()} additional wei.`)
        }

        const newTx = Transaction.create(
            tx.nonce,
            numberToBuffer(bufferToNumber(await self.node.db.get(self.Index(options.channelId))) + 1, Transaction.INDEX_LENGTH),
            currentValue.toBuffer('be', Transaction.VALUE_LENGTH),
            secp256k1.publicKeyCreate(channelKey)
        ).sign(self.node.peerInfo.id)

        await self.node.db.batch()
            .put(self.CurrentValue(options.channelId), currentValue.toBuffer('be', Transaction.VALUE_LENGTH))
            .put(self.Index(options.channelId), newTx.index)
            .put(self.ChannelKey(options.channelId), channelKey)
            .write()

        return newTx
    }

    return (options) => {

        let queue = queues.get(options.channelId.toString('base64'))
        if (!queue) {
            queue = new Queue(1, Infinity)
            queues.set(options.channelId.toString('base64'), queue)
        }

        return queue.add(() => transfer(options))
    }
}