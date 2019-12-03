'use strict'

const Queue = require('promise-queue')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { randomBytes } = require('crypto')

const chalk = require('chalk')

const { fromWei } = require('web3-utils')
const { isPartyA, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer, log, addPubKey, getId } = require('../utils')
const Transaction = require('../transaction')

module.exports = self => {
    const queues = new Map()

    /**
     * Handles records of previously opened or half-opened channels.
     *
     * @param {Object} record record that is stored in the database
     * @param {PeerId} to peerId of the counterparty
     */
    async function handlePreviousRecord(record, to) {
        switch (record.state) {
            case self.TransactionRecordState.INITIALIZED:
                console.log(`Opening channel ${chalk.yellow(channelId.toString('hex'))} with a previously signed restore transaction.`)
                return self.open(to, record.restoreTransaction)
            case self.TransactionRecordState.PRE_OPENED:
                record.nonce = randomBytes(Transaction.NONCE_LENGTH)
                record.state = self.TransactionRecordState.OPEN

                return record
            case self.TransactionRecordState.SETTLING:
            case self.TransactionRecordState.SETTLED:
                await self.handleClosedChannel(channelId, true)
                return self.open(to)
            default:
                return record
        }
    }

    /**
     * Fetches the current state from the database and updates it according to the
     * transferred amount. In case there is no open channel, the method will initiate a
     * payment channel opening.
     *
     * @param {PeerId} to receiver of the payment
     * @param {BN} amount amount of funds
     * @param {Buffer} channelId ID of the payment channel
     *
     * @returns {Promise<Transaction>} an update transaction for the payment channel.
     */
    async function transfer(to, amount, channelId, channelKey) {
        let record,
            recordExists = false

        try {
            record = await self.state(channelId)
            recordExists = true
        } catch (err) {
            if (err.notFound) {
                record = await self.open(to)
            } else {
                throw err
            }
        }

        if (recordExists) {
            record = await handlePreviousRecord(record, to)
        }

        const challenges = [secp256k1.publicKeyCreate(channelKey)]
        const previousChallenges = await self.node.paymentChannels.getPreviousChallenges(channelId)

        if (previousChallenges) challenges.push(previousChallenges)
        if (record.channelKey) challenges.push(secp256k1.publicKeyCreate(record.channelKey))

        const newTx = Transaction.create(
            record.nonce,
            numberToBuffer(bufferToNumber(record.currentIndex) + 1, Transaction.INDEX_LENGTH),
            getNewChannelBalance(record, to, amount).toBuffer('be', Transaction.VALUE_LENGTH),
            secp256k1.publicKeyCombine(challenges)
        ).sign(self.node.peerInfo.id)

        record.currentIndex = newTx.index
        record.currentOffchainBalance = newTx.value

        log(
            self.node.peerInfo.id,
            `Created tx with index ${chalk.cyan(record.currentIndex.toString('hex'))} on channel ${chalk.yellow(channelId.toString('hex'))}.`
        )

        record.channelKey = secp256k1.privateKeyTweakAdd(channelKey, record.channelKey || Buffer.alloc(32, 0))

        await self.setState(channelId, record)

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
