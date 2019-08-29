'use strict'

const EventEmitter = require('events')

const Web3 = require('web3')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const chalk = require('chalk')

const { pubKeyToEthereumAddress, pubKeyToPeerId, sendTransaction, log, compileIfNecessary, isPartyA, mineBlock, bufferToNumber } = require('../utils')

const open = require('./rpc/open')
const closingListener = require('./eventListeners/close')
const openingListener = require('./eventListeners/open')
const transfer = require('./transfer')
const registerHandlers = require('./handlers')
const Transaction = require('../transaction')

const HASH_LENGTH = 32
const CHANNEL_ID_LENGTH = HASH_LENGTH
const CHALLENGE_LENGTH = 33
const PRIVATE_KEY_LENGTH = 32
const COMPRESSED_PUBLIC_KEY_LENGTH = 33

const PREFIX = Buffer.from('payments-')
const PREFIX_LENGTH = PREFIX.length

const { PROTOCOL_SETTLE_CHANNEL } = require('../constants')

const SETTLEMENT_TIMEOUT = 40000

const CHANNEL_STATE_UNINITIALIZED = 0
const CHANNEL_STATE_FUNDED = 3
const CHANNEL_STATE_WITHDRAWABLE = 4

const fs = require('fs')
const path = require('path')
const protons = require('protons')

const { SettlementRequest, SettlementResponse } = protons(fs.readFileSync(path.resolve(__dirname, 'protos/messages.proto')))

// payments
// -> channelId
// ---> tx
// ---> restoreTx
// ---> index
// ---> key
// ---> currentValue
// ---> totalBalance
// ---> challenges -> keyHalf
// -> signatureHash
// ---> channelId

class PaymentChannel extends EventEmitter {
    constructor(options) {
        super()

        this.nonce = options.nonce
        this.contract = options.contract
        this.contractAddress = process.env.CONTRACT_ADDRESS
        this.node = options.node
        this.web3 = options.web3

        this.open = open(this)
        this.closingListener = closingListener(this)
        this.openingListener = openingListener(this)
        this.transfer = transfer(this)

        this.subscriptions = new Map()
        this.closingSubscriptions = new Map()
        this.settleTimestamps = new Map()
    }

    /**
     * Creates and initializes a new PaymentChannel instance.
     * It will check whether there is an up-to-date ABI of the contract
     * and compiles the contract if that isn't the case.
     *
     * @param {Hopr} node a libp2p node instance
     */
    static async create(node) {
        const web3 = new Web3(process.env.PROVIDER)

        const [nonce, compiledContract] = await Promise.all([
            web3.eth.getTransactionCount(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), 'latest'),
            compileIfNecessary([`${process.cwd()}/contracts/HoprChannel.sol`], [`${process.cwd()}/build/contracts/HoprChannel.json`])
        ])

        registerHandlers(node)

        const abi = require('../../build/contracts/HoprChannel.json').abi

        const paymentChannel = new PaymentChannel({
            node,
            nonce,
            contract: new web3.eth.Contract(abi, process.env.CONTRACT_ADDRESS, {
                from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
            }),
            web3
        })

        await paymentChannel.registerEventListeners()

        return paymentChannel
    }

    /**
     * Registers listeners on-chain opening events and the closing events of all
     * payment channels found in the database.
     */
    async registerEventListeners() {
        const register = (query, fn) =>
            new Promise((resolve, reject) =>
                this.node.db
                    .createKeyStream(query)
                    .on('data', fn)
                    .on('error', reject)
                    .on('end', resolve)
            )

        await Promise.all([
            register(
                {
                    gt: this.RestoreTransaction(Buffer.alloc(32, 0)),
                    lt: this.RestoreTransaction(Buffer.alloc(32, 255))
                },
                key => this.registerSettlementListener(key.slice(key.length - 32))
            ),
            register(
                {
                    gt: this.StashedRestoreTransaction(Buffer.alloc(32, 0)),
                    lt: this.StashedRestoreTransaction(Buffer.alloc(32, 255))
                },
                key => this.registerOpeningListener(key.slice(key.length - 32))
            )
        ])
    }

    /**
     * Registers a listener to the on-chain ClosedChannel event of a payment channel.
     *
     * @param {Buffer} channelId ID of the channel
     * @param {Function} listener function that is called whenever the `ClosedChannel` event
     * is fired.
     */
    registerSettlementListener(channelId, listener = this.closingListener) {
        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_LENGTH)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to close event of channel \x1b[33m${channelId.toString('hex')}\x1b[0m`)

        this.closingSubscriptions.set(
            channelId.toString('hex'),
            this.web3.eth.subscribe(
                'logs',
                {
                    topics: [this.web3.utils.sha3(`ClosedChannel(bytes32,bytes16,uint256)`), `0x${channelId.toString('hex')}`]
                },
                listener
            )
        )
    }

    /**
     * Registers a listener to the on-chain OpenedChannel event of a payment channel.
     *
     * @param {Buffer} channelId ID of the channel
     * @param {Function} listener function that is called whenever the `OpenedChannel` event
     * is fired.
     */
    registerOpeningListener(channelId, listener = this.openingListener) {
        if (typeof listener !== 'function')
            throw Error(`Please specify a function that is called when the close event is triggered. Got ${typeof listener} instead.`)

        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_LENGTH)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to opening event of channel \x1b[33m${channelId.toString('hex')}\x1b[0m`)

        this.contract.once(
            'OpenedChannel',
            {
                topics: [this.web3.utils.sha3(`OpenedChannel(bytes32,uint256,uint256)`), `0x${channelId.toString('hex')}`]
            },
            listener
        )
    }

    onceClosed(channelId, fn) {
        this.once(`closed ${channelId.toString('hex')}`, fn)
    }

    emitClosed(channelId, receivedMoney) {
        this.emit(`closed ${channelId.toString('hex')}`, receivedMoney)
    }

    ChannelKey(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('key-'), channelId], PREFIX_LENGTH + 4 + CHANNEL_ID_LENGTH)
    }

    Transaction(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('tx-'), channelId], PREFIX_LENGTH + 3 + CHANNEL_ID_LENGTH)
    }

    RestoreTransaction(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('restoreTx-'), channelId], PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH)
    }

    StashedRestoreTransaction(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('stashedRestoreTx-'), channelId], PREFIX_LENGTH + 17 + CHANNEL_ID_LENGTH)
    }

    Index(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('index-'), channelId], PREFIX_LENGTH + 6 + CHANNEL_ID_LENGTH)
    }

    CurrentValue(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('currentValue-'), channelId], PREFIX_LENGTH + 13 + CHANNEL_ID_LENGTH)
    }

    InitialValue(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('initialBalance-'), channelId], PREFIX_LENGTH + 15 + CHANNEL_ID_LENGTH)
    }

    CurrentOnChainBalance(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('onChainBalance-'), channelId], PREFIX_LENGTH + 15 + CHANNEL_ID_LENGTH)
    }

    TotalBalance(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('totalBalance-'), channelId], PREFIX_LENGTH + 13 + CHANNEL_ID_LENGTH)
    }

    Challenge(channelId, challenge) {
        return Buffer.concat([PREFIX, Buffer.from('challenge-'), channelId, challenge], PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH + CHALLENGE_LENGTH)
    }

    ChannelId(signatureHash) {
        return Buffer.concat([PREFIX, Buffer.from('channelId-'), signatureHash], PREFIX_LENGTH, PREFIX_LENGTH + 10 + HASH_LENGTH)
    }

    /**
     * Fetches the previous challenges from the database and add them together.
     *
     * @param {Buffer} channelId ID of the payment channel
     */
    getPreviousChallenges(channelId) {
        return new Promise(async (resolve, reject) => {
            let buf
            try {
                buf = secp256k1.publicKeyCreate(await this.node.db.get(this.ChannelKey(channelId)))
            } catch (err) {
                if (!err.notFound) throw err
            }

            this.node.db
                .createReadStream({
                    gt: this.Challenge(channelId, Buffer.alloc(PRIVATE_KEY_LENGTH, 0)),
                    lt: this.Challenge(channelId, Buffer.alloc(PRIVATE_KEY_LENGTH, 255))
                })
                .on('data', obj => {
                    const challenge = obj.key.slice(
                        PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH,
                        PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH + COMPRESSED_PUBLIC_KEY_LENGTH
                    )
                    const ownKeyHalf = obj.value

                    const pubKeys = [challenge, secp256k1.publicKeyCreate(ownKeyHalf)]

                    if (buf) {
                        pubKeys.push(buf)
                    }

                    buf = secp256k1.publicKeyCombine(pubKeys)
                })
                .on('error', reject)
                .on('end', () => resolve(buf))
        })
    }

    /**
     * Returns a promise that resolves just when the funds from the channel are withdrawn.
     *
     * @notice When using this method with `process.env.NETWORK === 'ganache'`, this method
     * will ask Ganache to mine blocks and increase the block time until the payment channel
     * becomes withdrawable.
     *
     * @param {Buffer} channelId ID of the channel
     */
    withdraw(channelId) {
        const self = this

        /**
         * Submits a withdraw transaction and cleans up attached event listeners.
         */
        const withdraw = async () => {
            const restoreTx = Transaction.fromBuffer(await this.node.db.get(this.RestoreTransaction(channelId)))

            return self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(restoreTx.counterparty))).then(receipt => {
                const subscription = self.subscriptions.get(channelId.toString('hex'))
                if (subscription) {
                    subscription.unsubscribe()
                    self.subscriptions.delete(channelId.toString('hex'))
                }

                const closingSubscription = self.closingSubscriptions.get(channelId.toString('hex'))
                if (closingSubscription) {
                    closingSubscription.unsubscribe()
                    self.closingSubscriptions.delete(channelId.toString())
                }

                self.deleteChannel(channelId)

                return receipt
            })
        }

        /**
         * Returns a promise that returns just when the channel is withdrawable.
         */
        const waitUntilChannelIsWithdrawable = () => {
            return new Promise(async (resolve, reject) => {
                const [channel, blockTimestamp] = await Promise.all([
                    self.contract.methods.channels(channelId).call(
                        {
                            from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
                        },
                        'latest'
                    ),
                    self.web3.eth.getBlock('latest', false).then(block => new BN(block.timestamp))
                ])

                if (channel.state == CHANNEL_STATE_WITHDRAWABLE && blockTimestamp.gt(new BN(channel.settleTimestamp))) return resolve()

                self.settleTimestamps.set(channelId.toString('hex'), new BN(channel.settleTimestamp))
                const subscription = self.web3.eth
                    .subscribe('newBlockHeaders')
                    .on('error', err => reject(err))
                    .on('data', block => {
                        const blockTimestamp = new BN(block.timestamp)
                        log(self.node.peerInfo.id, `Waiting ... Block ${block.number}.`)

                        if (blockTimestamp.gt(self.settleTimestamps.get(channelId.toString('hex')))) {
                            subscription.unsubscribe((err, ok) => {
                                if (err) return reject(err)

                                if (ok) resolve()
                            })
                        } else if (process.env.NETWORK === 'ganache') {
                            // ================ Only for testing ================
                            mineBlock(self.contract.currentProvider)
                            // ==================================================
                        }
                    })

                self.subscriptions.set(channelId.toString('hex'), subscription)
                if (process.env.NETWORK === 'ganache') {
                    // ================ Only for testing ================
                    mineBlock(self.contract.currentProvider)
                    // ==================================================
                }
            })
        }

        return waitUntilChannelIsWithdrawable()
            .then(() => withdraw())
            .then(() => self.node.db.get(self.CurrentOnChainBalance(channelId)))
            .then(balance => new BN(balance))
    }

    /**
     * Returns a promise that resolves just when a settlement transaction were successfully
     * submitted to the Ethereum network.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Transaction} [tx] tx that is used to close the payment channel
     */
    submitSettlementTransaction(channelId, tx) {
        return new Promise(async (resolve, reject) => {
            let channelKey

            if (!tx) {
                try {
                    tx = await this.getLastTransaction(channelId)
                } catch (err) {
                    reject(err)
                }
            }

            log(this.node.peerInfo.id, `Trying to close payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m. Nonce is ${this.nonce}`)

            this.contractCall(
                this.contract.methods.closeChannel(
                    tx.index,
                    tx.nonce,
                    new BN(tx.value).toString(),
                    tx.curvePoint.slice(0, 32),
                    tx.curvePoint.slice(32, 33),
                    tx.signature.slice(0, 32),
                    tx.signature.slice(32, 64),
                    bufferToNumber(tx.recovery) + 27
                )
            )
                .then(receipt => {
                    log(
                        this.node.peerInfo.id,
                        `Settled channel \x1b[33m${channelId.toString('hex')}\x1b[0m with txHash \x1b[32m${
                            receipt.transactionHash
                        }\x1b[0m. Nonce is now \x1b[31m${this.nonce}\x1b[0m`
                    )
                    return resolve(receipt)
                })
                .catch(err => reject(err))
        })
    }

    /**
     * Returns a promise that resolves with the latest transaction that exists in the database.
     * Search order:
     *  1. latest update transaction
     *  2. restore transaction
     *  3. stashed restore transaction (in case there is one)
     *
     * @param {Bufer} channelId ID of the payment channel
     */
    getLastTransaction(channelId) {
        return this.node.db
            .get(this.Transaction(channelId))
            .catch(err => {
                if (!err.notFound) throw err

                return this.node.db.get(this.RestoreTransaction(channelId)).catch(err => {
                    if (!err.notFound) throw err

                    return this.node.db.get(this.StashedRestoreTransaction(channelId)).catch(err => {
                        if (!err.notFound) throw err
                    })
                })
            })
            .then(txBuffer => Transaction.fromBuffer(txBuffer))
            .catch(err => {
                if (err.notFound) {
                    throw Error(`Haven't found any transaction for channel ${chalk.yellow(channelId.toString('hex'))}.`)
                } else {
                    throw err
                }
            })
    }

    /**
     * Returns a promise that resolves just when all database entries related to the
     * given channelId are deleted.
     *
     * @param {Buffer} channelId ID of the payment channel
     */
    deleteChannel(channelId) {
        return new Promise((resolve, reject) => {
            let batch = this.node.db
                .batch()
                .del(this.ChannelKey(channelId))
                .del(this.Transaction(channelId))
                .del(this.RestoreTransaction(channelId))
                .del(this.StashedRestoreTransaction(channelId))
                .del(this.Index(channelId))
                .del(this.CurrentValue(channelId))
                .del(this.CurrentOnChainBalance(channelId))
                .del(this.InitialValue(channelId))
                .del(this.TotalBalance(channelId))

            this.node.db
                .createKeyStream({
                    gt: this.Challenge(channelId, Buffer.alloc(COMPRESSED_PUBLIC_KEY_LENGTH, 0)),
                    lt: this.Challenge(channelId, Buffer.alloc(COMPRESSED_PUBLIC_KEY_LENGTH, 255))
                })
                .on('data', key => {
                    // console.log(key.toString())
                    batch = batch.del(key)
                })
                .on('end', () => resolve(batch.write()))
                .on('err', reject)
        })
    }

    /**
     * Computes the delta of funds that were received with the given transaction in relation to the
     * initial balance.
     *
     * @param {Transaction} receivedTx the transaction upon which the delta funds is computed
     * @param {PeerId} counterparty peerId of the counterparty that is used to decide which side of
     * payment channel we are, i. e. party A or party B.
     * @param {Buffer} currentValue the currentValue of the payment channel.
     */
    getEmbeddedMoney(receivedTx, counterparty, currentValue) {
        currentValue = new BN(currentValue)
        const newValue = new BN(receivedTx.value)

        const self = pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        const otherParty = pubKeyToEthereumAddress(counterparty.pubKey.marshal())

        if (isPartyA(self, otherParty)) {
            return newValue.isub(currentValue)
        } else {
            return currentValue.isub(newValue)
        }
    }

    async counterpartyHasMoreRecentTransaction(channelId) {
        const [tx, channelIndex] = await Promise.all([this.getLastTransaction(channelId), this.node.db.get(this.Index(channelId)).then(index => new BN(index))])
        return channelIndex.gt(new BN(tx.index))
    }

    /**
     * Asks the counterparty of the given channelId to provide the latest transaction.
     *
     * @param {Buffer[]} channelIds ID of the payment channel
     * @return {Promise} a promise that resolves with the latest transaction of the
     * counterparty and rejects if it is invalid and/or outdated.
     */
    getLatestTransactionFromCounterparty(channelIds) {
        if (!Array.isArray(channelIds)) channelIds = [channelIds]

        const queryNode = channelId => new Promise(async (resolve, reject) => {
            const restoreTx = Transaction.fromBuffer(await this.node.db.get(this.RestoreTransaction(channelId)))
            const counterparty = await pubKeyToPeerId(restoreTx.counterparty)

            log(this.node.peerInfo.id, `Asking node ${chalk.blue(counterparty.toB58String())} to send latest update transaction.`)

            let conn
            try {
                conn = await this.node.peerRouting.findPeer(counterparty).then(peerInfo => this.node.dialProtocol(peerInfo, PROTOCOL_SETTLE_CHANNEL))
            } catch (err) {
                return reject(chalk.red(err.message))
            }

            pull(
                pull.once(SettlementRequest.encode({
                    channelId
                })),
                lp.encode(),
                conn,
                lp.decode(),
                pull.drain(buf => {
                    let response

                    try {
                        response = SettlementResponse.decode(buf)
                    } catch (err) {
                        reject(Error(
                            `Counterparty ${chalk.blue(counterparty.toB58String())} didn't send a valid response to close channel ${chalk.yellow(
                                channelId.toString('hex')
                            )}.`
                        ))
                    }

                    const tx = Transaction.fromBuffer(response.transaction)

                    if (!tx.verify(counterparty)) return reject(Error(`Invalid transaction on channel ${chalk.yellow(channelId.toString('hex'))}.`))

                    // @TODO do plausibility checks

                    resolve(tx)
                    return false
                })
            )

        })

        return Promise.all(channelIds.map(channelId => queryNode(channelId)))
    }

    closeChannel(channelId) {
        return new Promise(async (resolve, reject) => {
            const channel = await this.contract.methods.channels(channelId).call(
                {
                    from: pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
                },
                'latest'
            )

            switch (parseInt(channel.state)) {
                case CHANNEL_STATE_UNINITIALIZED:
                    await this.deleteChannel(channelId)

                    return reject(Error(`Channel ${chalk.yellow(channelId.toString('hex'))} doesn't exist.`))
                case CHANNEL_STATE_FUNDED:
                    let lastTx
                    if (await this.counterpartyHasMoreRecentTransaction(channelId)) {
                        lastTx = await new Promise((resolve, reject) => {
                            const timeout = setTimeout(resolve, SETTLEMENT_TIMEOUT)
                            this.getLatestTransactionFromCounterparty([channelId])
                                .then(txs => {
                                    clearTimeout(timeout)
                                    resolve(txs[0])
                                })
                                .catch(err => {
                                    clearTimeout(timeout)
                                    console.log(chalk.red(err.message))
                                    resolve()
                                })
                        })
                    }

                    this.onceClosed(channelId, () => resolve(this.withdraw(channelId)))
                    this.submitSettlementTransaction(channelId, lastTx)
                    break
                case CHANNEL_STATE_WITHDRAWABLE:
                    resolve(this.withdraw(channelId))
                    break
                default:
                    log(this.node.peerInfo.id, `Channel in unknown state: channel.state = ${chalk.red(channel.state)}.`)

                    reject(new BN(0))
            }
        })
    }

    closeChannels() {
        return new Promise((resolve, reject) => {
            const promises = []
            this.node.db
                .createKeyStream({
                    gt: this.RestoreTransaction(Buffer.alloc(CHANNEL_ID_LENGTH, 0)),
                    lt: this.RestoreTransaction(Buffer.alloc(CHANNEL_ID_LENGTH, 255))
                })
                .on('error', err => reject(err))
                .on('data', key => {
                    promises.push(
                        /* prettier-ignore */
                        this.closeChannel(key.slice(key.length - CHANNEL_ID_LENGTH)).catch(err => new BN(0))
                    )
                })
                .on('end', () => {
                    if (promises.length > 0) {
                        resolve(Promise.all(promises).then(results => results.reduce((acc, value) => acc.iadd(value))))
                    } else {
                        resolve(new BN(0))
                    }
                })
        })
    }

    /**
     * Takes a transaction object generetad by web3.js and publishes it in the
     * network. It automatically determines the necessary amount of gas i
     *
     * @param {Object} txObject the txObject generated by web3.js
     * @param {String} value amount of Ether that is sent along with the transaction
     * @param {Function} cb the function to be called afterwards
     */
    async contractCall(txObject, value, cb) {
        if (typeof value === 'function') {
            cb = value
            value = '0'
        }

        if (!value) value = '0'

        const estimatedGas = await txObject.estimateGas({
            from: pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        })

        this.nonce = this.nonce + 1

        const promise = sendTransaction(
            {
                to: this.contractAddress,
                nonce: this.nonce - 1,
                gas: estimatedGas,
                data: txObject.encodeABI()
            },
            this.node.peerInfo.id,
            this.web3
        )

        if (typeof cb === 'function') {
            promise.then(receipt => cb(null, receipt)).catch(cb)
        } else {
            return promise
        }
    }
}

module.exports = PaymentChannel
