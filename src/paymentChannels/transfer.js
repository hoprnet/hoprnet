'use strict'

const Queue = require('promise-queue')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { isPartyA, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer, log } = require('../utils')
const Transaction = require('../transaction')

module.exports = (self) => {
    const queues = new Map()

    /**
     * Computes the new balance of the channel.
     * 
     * @param {Buffer} channelId ID of the channel
     * @param {PeerId} to peerId of the recipient
     * @param {BN} amount of funds to transfer
     */
    async function getNewChannelBalance(channelId, to, amount) {
        const currentValue = new BN(await self.node.db.get(self.CurrentValue(channelId)))

        const partyA = isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal()))

        if (partyA) {
            currentValue.isub(amount)
            if (currentValue.isNeg())
                throw Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.abs().toString()} additional wei.`)
        } else {
            currentValue.iadd(amount)

            const totalBalance = new BN(await self.node.db.get(self.TotalBalance(channelId)))
            if (currentValue.gt(totalBalance))
                throw Error(`Insufficient funds. Please equip the payment channel with at least ${currentValue.sub(totalBalance).toString()} additional wei.`)
        }

        return currentValue
    }

    async function transfer(options) {
        let tx, channelKey = options.key

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

        const [newBalance, previousChallenges, index] = await Promise.all([
            getNewChannelBalance(options.channelId, options.to, options.amount),
            self.getPreviousChallenges(options.channelId),
            self.node.db.get(self.Index(options.channelId))
        ])

        const pubKeys = [
            secp256k1.publicKeyCreate(options.key)
        ]

        if (previousChallenges) {
            pubKeys.push(previousChallenges)
        }

        const newTx = Transaction.create(
            tx.nonce,
            numberToBuffer(bufferToNumber(index) + 1, Transaction.INDEX_LENGTH),
            newBalance.toBuffer('be', Transaction.VALUE_LENGTH),
            secp256k1.publicKeyCombine(pubKeys)
        ).sign(self.node.peerInfo.id)

        log(self.node.peerInfo.id, `Created tx with index ${numberToBuffer(bufferToNumber(index) + 1, Transaction.INDEX_LENGTH).toString('hex')} on channel ${options.channelId.toString('hex')}.`)

        try {
            channelKey = secp256k1.privateKeyTweakAdd(channelKey, await self.node.db.get(self.ChannelKey(options.channelId)))
        } catch (err) {
            if (!err.notFound)
                throw err
        }

        await self.node.db.batch()
            .put(self.CurrentValue(options.channelId), newBalance.toBuffer('be', Transaction.VALUE_LENGTH))
            .put(self.Index(options.channelId), newTx.index)
            .put(self.ChannelKey(options.channelId), channelKey)
            .write({ sync: true })

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