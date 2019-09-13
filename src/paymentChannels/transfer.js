'use strict'

const Queue = require('promise-queue')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const chalk = require('chalk')

const { fromWei } = require('web3-utils')
const { isPartyA, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer, log, addPubKey, getId } = require('../utils')
const Transaction = require('../transaction')

module.exports = self => {
    const queues = new Map()

    /**
     * Fetches the current state from database and updates it according to the amount
     * transferred. In case there is no open channel, the method will initiate a
     * payment channel opening.
     *
     * @param {PeerId} to receiver of the payment
     * @param {BN} amount amount of funds
     * @param {Buffer} channelId ID of the payment channel
     *
     * @returns {Promise<Transaction>} an update transaction for the payment channel.
     */
    async function transfer(to, amount, channelId, channelKey) {
        let record
        try {
            record = await self.state(channelId)
        } catch (err) {
            if (err.notFound) {
                await self.open(to)
                record = await self.state(channelId)
            } else {
                throw err
            }
        }

        switch (record.state) {
            case self.TransactionRecordState.INITIALIZED:
                console.log(`Opening channel ${chalk.yellow(channelId.toString('hex'))} with a previously signed restore transaction.`)
                await self.open(to, record.restoreTransaction)
                record = await self.state(channelId)
                break
            case self.TransactionRecordState.SETTLING:
            case self.TransactionRecordState.SETTLED:
                await self.handleClosedChannel(channelId, true)
                await self.open(to)
                record = await self.state(channelId)
                break
        }

        const challenges = [secp256k1.publicKeyCreate(channelKey)]
        const previousChallenges = await self.node.paymentChannels.getPreviousChallenges(channelId)

        if (previousChallenges) challenges.push(previousChallenges)
        if (record.channelKey) challenges.push(secp256k1.publicKeyCreate(record.channelKey))

        const newTx = Transaction.create(
            record.lastTransaction.nonce,
            numberToBuffer(bufferToNumber(record.currentIndex) + 1, Transaction.INDEX_LENGTH),
            getNewChannelBalance(record, to, amount).toBuffer('be', Transaction.VALUE_LENGTH),
            // @TODO nach dem Initialisieren müsste das leer sein
            secp256k1.publicKeyCombine(challenges)
        ).sign(self.node.peerInfo.id)

        log(
            self.node.peerInfo.id,
            `Created tx with index ${chalk.cyan(
                numberToBuffer(bufferToNumber(record.currentIndex) + 1, Transaction.INDEX_LENGTH).toString('hex')
            )} on channel ${chalk.yellow(channelId.toString('hex'))}.`
        )

        try {
            channelKey = secp256k1.privateKeyTweakAdd(channelKey, record.channelKey || Buffer.alloc(32, 0))
        } catch (err) {
            if (!err.notFound) throw err
        }

        await self.setState(channelId, {
            currentOffchainBalance: newTx.value,
            currentIndex: newTx.index,
            channelKey
        })

        return newTx
    }

    /**
     * Computes the new balance of the channel.
     *
     * @param {Object} record current off-chain state
     * @param {PeerId} to peerId of the recipient
     * @param {BN} amount of funds to transfer
     */
    function getNewChannelBalance(record, to, amount) {
        const currentValue = new BN(record.currentOffchainBalance)

        const partyA = isPartyA(
            /* prettier-ignore */
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal())
        )

        if (partyA) {
            currentValue.isub(amount)
            if (currentValue.isNeg())
                throw Error(
                    `Insufficient funds. Please equip the payment channel with at least ${chalk.magenta(
                        `${fromWei(currentValue.abs(), 'ether').toString()} ETH`
                    )} additionally.`
                )
        } else {
            currentValue.iadd(amount)

            const totalBalance = new BN(record.totalBalance)
            if (currentValue.gt(totalBalance))
                throw Error(
                    `Insufficient funds. Please equip the payment channel with at least ${chalk.magenta(
                        `${fromWei(currentValue.sub(totalBalance), 'ether').toString()} ETH`
                    )} additionally.`
                )
        }

        return currentValue
    }

    return async (to, amount, channelKey) => {
        to = await addPubKey(to)

        const channelId = getId(pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), pubKeyToEthereumAddress(to.pubKey.marshal()))

        let queue = queues.get(channelId.toString('base64'))
        if (!queue) {
            queue = new Queue(1, Infinity)
            queues.set(channelId.toString('base64'), queue)
        }

        return queue.add(() => transfer(to, amount, channelId, channelKey))
    }
}
