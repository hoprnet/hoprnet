'use strict'

const EventEmitter = require('events')

const Web3 = require('web3')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const chalk = require('chalk')

const { pubKeyToEthereumAddress, pubKeyToPeerId, sendTransaction, log, compileIfNecessary, isPartyA } = require('../utils')

const open = require('./rpc/open')
const close = require('./rpc/close')
const withdraw = require('./rpc/withdraw')

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

const { ChannelState } = require('./enums.json')

const SETTLEMENT_TIMEOUT = 40000

const { PROTOCOL_SETTLE_CHANNEL } = require('../constants')

const fs = require('fs')
const path = require('path')
const protons = require('protons')

const { SettlementRequest, SettlementResponse } = protons(fs.readFileSync(path.resolve(__dirname, './protos/messages.proto')))
const { TransactionRecord, TransactionRecordState } = protons(fs.readFileSync(path.resolve(__dirname, './protos/transactionRecord.proto')))

// payments
// -> channelId
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
        this.closeChannel = close(this)
        this.withdraw = withdraw(this)

        this.eventListeners = require('./eventListeners')(this)

        this.transfer = transfer(this)

        this.subscriptions = new Map()
        this.closingSubscriptions = new Map()
        this.settleTimestamps = new Map()
    }

    get TransactionRecordState() {
        return TransactionRecordState
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

        const [nonce] = await Promise.all([
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

        paymentChannel.registerOpenedForListener()

        await paymentChannel.registerEventListeners()

        return paymentChannel
    }

    State(channelId) {
        return Buffer.concat([PREFIX, Buffer.from('record-'), channelId], PREFIX_LENGTH + 7 + CHANNEL_ID_LENGTH)
    }

    async setState(channelId, newState) {
        // console.log(chalk.blue(this.node.peerInfo.id.toB58String()), newState)

        let record = {}
        try {
            record = await this.state(channelId)
        } catch (err) {
            if (!err.notFound) throw err
        }

        Object.assign(record, newState)

        // if (!record.counterparty && record.state != this.TransactionRecordState.PRE_OPENED) throw Error(`no counterparty '${JSON.stringify(record)}'`)

        // if (!record.restoreTransaction && record.state != this.TransactionRecordState.PRE_OPENED)
        //     throw Error(`no restore transaction '${JSON.stringify(record)}'`)

        if (record.restoreTransaction) record.restoreTransaction = record.restoreTransaction.toBuffer()
        if (record.lastTransaction) record.lastTransaction = record.lastTransaction.toBuffer()

        return this.node.db.put(this.State(channelId), TransactionRecord.encode(record))
    }

    async state(channelId, encodedRecord) {
        if (!encodedRecord) {
            try {
                encodedRecord = await this.node.db.get(this.State(channelId))
            } catch (err) {
                if (err.notFound) {
                    err = Error(`Couldn't find any record for channel ${chalk.yellow(channelId.toString('hex'))}`)

                    err.notFound = true
                }

                throw err
            }
        }

        const record = TransactionRecord.decode(encodedRecord)

        if (record.restoreTransaction) record.restoreTransaction = Transaction.fromBuffer(record.restoreTransaction)
        if (record.lastTransaction) record.lastTransaction = Transaction.fromBuffer(record.lastTransaction)

        return record
    }

    /**
     * Deletes all information in the database corresponding to the given payment channel id.
     *
     * @param {Buffer} channelId ID of the payment channel
     */
    deleteState(channelId) {
        return new Promise(async (resolve, reject) => {
            log(this.node.peerInfo.id, `Deleting record for channel ${chalk.yellow(channelId.toString('hex'))}`)

            let batch = this.node.db.batch().del(this.State(channelId))

            this.node.db
                .createKeyStream({
                    gt: this.Challenge(channelId, Buffer.alloc(COMPRESSED_PUBLIC_KEY_LENGTH, 0x00)),
                    lt: this.Challenge(channelId, Buffer.alloc(COMPRESSED_PUBLIC_KEY_LENGTH, 0xff))
                })
                .on('data', key => {
                    // console.log(key.toString())
                    batch = batch.del(key)
                })
                .on('end', () => resolve(batch.write({ sync: true })))
                .on('err', reject)
        })
    }

    /**
     * Registers listeners on-chain opening events and the closing events of all
     * payment channels found in the database.
     */
    registerEventListeners() {
        return new Promise((resolve, reject) => {
            const settledChannels = []
            const openingChannels = []
            this.node.db
                .createReadStream({
                    gt: this.State(Buffer.alloc(CHANNEL_ID_LENGTH, 0x00)),
                    lt: this.State(Buffer.alloc(CHANNEL_ID_LENGTH, 0xff))
                })
                .on('data', ({ key, value }) => {
                    const record = TransactionRecord.decode(value)
                    const channelId = key.slice(key.length - CHANNEL_ID_LENGTH)

                    switch (record.state) {
                        case this.TransactionRecordState.UNITIALIZED:
                        case this.TransactionRecordState.OPENING:
                            openingChannels.push(
                                (async () => {
                                    const networkState = await this.contract.methods.channels(channelId).call({
                                        from: pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
                                    })

                                    switch (networkState.state) {
                                        case ChannelState.ACTIVE:
                                            record.state = this.TransactionRecordState.OPEN
                                            record.currentIndex = networkState.index
                                            record.initialBalance = state.restoreTransaction.value

                                            record.currentOffchainBalance = new BN(state.balanceA).toBuffer('be', Transaction.VALUE_LENGTH)
                                            record.currentOnchainBalance = new BN(state.balanceA).toBuffer('be', Transaction.VALUE_LENGTH)
                                            record.totalBalance = new BN(event.returnValues.amount).toBuffer('be', Transaction.VALUE_LENGTH)

                                            if (!record.lastTransaction) record.lastTransaction = state.restoreTransaction

                                            await this.setState(channelId, record)
                                        default:
                                            this.registerOpeningListener(channelId)
                                    }
                                })()
                            )
                            break
                        case this.TransactionRecordState.PRE_OPENED:
                        case this.TransactionRecordState.OPEN:
                            this.registerSettlementListener(channelId)
                            break
                        case this.TransactionRecordState.SETTLING:
                        case this.TransactionRecordState.SETTLED:
                            settledChannels.push({
                                channelId,
                                state: record
                            })
                            break
                        default:
                            console.log(`Found record for channel ${channelId.toString('hex')} entry for with state set to '${record.state}'.`)
                    }
                })
                .on('error', reject)
                .on('end', () => {
                    const promises = []

                    if (openingChannels.length > 0) promises.push(openingChannels)
                    if (settledChannels.length > 0) promises.push(settledChannels.map(this.handleClosedChannel.bind(this)))

                    if (promises.length > 0) return resolve(Promise.all(promises))

                    resolve()
                })
        })
    }

    handleClosedChannel(settledChannel, autoWithdraw = false) {
        return this.contract.methods
            .channels(settledChannel.channelId)
            .call(
                {
                    from: pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
                },
                'latest'
            )
            .then(channelState => {
                switch (parseInt(channelState.state)) {
                    case ChannelState.UNINITIALIZED:
                        return this.deleteState(settledChannel.channelId)
                    case ChannelState.ACTIVE:
                        const state = {
                            state: this.TransactionRecordState.PRE_OPENED,
                            currentIndex: new BN(1).toBuffer('be', Transaction.INDEX_LENGTH),
                            initialBalance: new BN(channelState.balanceA).toBuffer('be', Transaction.VALUE_LENGTH),
                            currentOffchainBalance: new BN(channelState.balanceA).toBuffer('be', Transaction.VALUE_LENGTH),
                            currentOnchainBalance: new BN(channelState.balanceA).toBuffer('be', Transaction.VALUE_LENGTH),
                            totalBalance: new BN(channelState.balance).toBuffer('be', Transaction.VALUE_LENGTH),
                            preOpened: true
                        }

                        this.registerSettlementListener(settledChannel.channelId)

                        // this.emitOpened(settledChannel.channelId, state)

                        return this.deleteState(settledChannel.channelId).then(() => this.setState(settledChannel.channelId, state))
                    case ChannelState.PENDING_SETTLEMENT:
                        if (autoWithdraw)
                            return this.withdraw(channelState, localState).then(() =>
                                // @TODO this is probably not the intended functionality
                                this.setState(settledChannel.channelId, {
                                    state: this.TransactionRecordState.WITHDRAWABLE
                                })
                            )

                        return this.setState(settledChannel.channelId, {
                            state: this.TransactionRecordState.WITHDRAWABLE
                        })
                    default:
                        throw Error(`Invalid state. Got ${channelState.state}`)
                }
            })
    }
    /**
     * Registers a listener to the on-chain ClosedChannel event of a payment channel.
     *
     * @param {Buffer} channelId ID of the channel
     * @param {Function} listener function that is called whenever the `ClosedChannel` event
     * is fired.
     */
    registerSettlementListener(channelId, listener = this.eventListeners.closingListener) {
        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_LENGTH)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to close event of channel ${chalk.yellow(channelId.toString('hex'))}`)

        const eventName = 'ClosedChannel'
        const path = '../../build/contracts/HoprChannel.json'
        const [eventABI] = require('../../build/contracts/HoprChannel.json').abi.filter(obj => obj.name == eventName)

        if (!eventABI) throw Error(`Found no ABI definition for event '${eventName}' in '${require('path').resolve(__dirname, path)}'.`)

        this.closingSubscriptions.set(
            channelId.toString('hex'),
            this.contract.events.ClosedChannel(
                {
                    topics: [eventABI.signature, `0x${channelId.toString('hex')}`]
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
    registerOpeningListener(channelId, listener = this.eventListeners.openingListener) {
        if (typeof listener !== 'function')
            throw Error(`Please specify a function that is called when the close event is triggered. Got ${typeof listener} instead.`)

        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_LENGTH)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to opening event of channel ${chalk.yellow(channelId.toString('hex'))}`)

        this.contract.once(
            'OpenedChannel',
            {
                topics: [this.web3.utils.sha3(`OpenedChannel(bytes32,uint256,uint256)`), `0x${channelId.toString('hex')}`]
            },
            listener
        )
    }

    registerOpenedForListener(listener = this.eventListeners.openedForListener) {
        const ownTopic = pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal()).replace(/(0x)([0-9a-fA-F]{20})/, '$1000000000000000000000000$2').toLowerCase()

        const eventName = 'OpenedChannelFor'
        const path = '../../build/contracts/HoprChannel.json'
        const [eventABI] = require('../../build/contracts/HoprChannel.json').abi.filter(obj => obj.name == eventName)

        if (!eventABI) throw Error(`Found no ABI definition for event '${eventName}' in '${require('path').resolve(__dirname, path)}'.`)

        this.contract.events.OpenedChannelFor(
            {
                topics: [eventABI.signature, ownTopic, null]
            },
            listener
        )

        this.contract.events.OpenedChannelFor(
            {
                topics: [eventABI.signature, null, ownTopic]
            },
            listener
        )
    }

    onceOpened(channelId, fn) {
        return this.once(OpenEvent(channelId), fn)
    }

    emitOpened(channelId, state) {
        this.setState(channelId, state)

        if (this.listenerCount(OpenEvent(channelId)) > 0) {
            this.emit(OpenEvent(channelId), state)
        }
    }

    onceClosed(channelId, fn) {
        return this.once(CloseEvent(channelId), fn)
    }

    emitClosed(channelId, state) {
        this.setState(channelId, state)

        if (this.listenerCount(CloseEvent(channelId)) > 0) {
            this.emit(CloseEvent(channelId), state)
        }
    }

    Challenge(channelId, challenge) {
        return Buffer.concat([PREFIX, Buffer.from('challenge-'), channelId, challenge], PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH + CHALLENGE_LENGTH)
    }

    ChannelId(signatureHash) {
        return Buffer.concat([PREFIX, Buffer.from('channelId-'), signatureHash], PREFIX_LENGTH, PREFIX_LENGTH + 10 + HASH_LENGTH)
    }

    /**
     * Fetches the previous challenges from the database and sum them up.
     *
     * @param {Buffer} channelId ID of the payment channel
     */
    async getPreviousChallenges(channelId) {
        let pubKeys = []

        return new Promise((resolve, reject) =>
            this.node.db
                .createReadStream({
                    gt: this.Challenge(channelId, Buffer.alloc(PRIVATE_KEY_LENGTH, 0x00)),
                    lt: this.Challenge(channelId, Buffer.alloc(PRIVATE_KEY_LENGTH, 0xff))
                })
                .on('data', ({ key, value }) => {
                    const challenge = key.slice(PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH, PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH + COMPRESSED_PUBLIC_KEY_LENGTH)
                    const ownKeyHalf = value

                    pubKeys.push(secp256k1.publicKeyCombine(challenge, secp256k1.publicKeyCreate(ownKeyHalf)))
                })
                .on('error', reject)
                .on('end', () => {
                    if (pubKeys.length > 0) return resolve(secp256k1.publicKeyCombine(pubKeys))

                    resolve()
                })
        )
    }

    /**
     * Computes the delta of funds that were received with the given transaction in relation to the
     * initial balance.
     *
     * @param {Buffer} newValue the transaction upon which the delta funds is computed
     * @param {Buffer} currentValue the currentValue of the payment channel.
     * @param {PeerId} counterparty peerId of the counterparty that is used to decide which side of
     * payment channel we are, i. e. party A or party B.
     */
    getEmbeddedMoney(newValue, currentValue, counterparty) {
        currentValue = new BN(currentValue)
        newValue = new BN(newValue)

        const self = pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        const otherParty = pubKeyToEthereumAddress(counterparty.pubKey.marshal())

        if (isPartyA(self, otherParty)) {
            return newValue.isub(currentValue)
        } else {
            return currentValue.isub(newValue)
        }
    }

    /**
     * Asks the counterparty of the given channelId to provide the latest transaction.
     *
     * @param {Object || Object[]} channels
     * @param {Buffer} channel.channelId
     * @param {Object} channel.state
     *
     * @returns {Promise} a promise that resolves with the latest transaction of the
     * counterparty and rejects if it is invalid and/or outdated.
     */
    getLatestTransactionFromCounterparty(channelId, state) {
        return new Promise(async (resolve, reject) => {
            const counterparty = await pubKeyToPeerId(state.counterparty)

            log(
                this.node.peerInfo.id,
                `Asking node ${chalk.blue(counterparty.toB58String())} to send latest update transaction for channel ${chalk.yellow(
                    channelId.toString('hex')
                )}.`
            )

            let conn
            try {
                conn = await this.node.peerRouting.findPeer(counterparty).then(peerInfo => this.node.dialProtocol(peerInfo, PROTOCOL_SETTLE_CHANNEL))
            } catch (err) {
                return reject(chalk.red(err.message))
            }

            const timeout = setTimeout(reject, SETTLEMENT_TIMEOUT)

            pull(
                pull.once(
                    SettlementRequest.encode({
                        channelId
                    })
                ),
                lp.encode(),
                conn,
                lp.decode(),
                pull.drain(buf => {
                    if (!buf || !Buffer.isBuffer(buf) || buf.length == 0) {
                        clearTimeout(timeout)
                        reject()
                        return false
                    }

                    let response
                    try {
                        response = SettlementResponse.decode(buf)
                    } catch (err) {
                        clearTimeout(timeout)
                        return reject(
                            Error(
                                `Counterparty ${chalk.blue(counterparty.toB58String())} didn't send a valid response to close channel ${chalk.yellow(
                                    channelId.toString('hex')
                                )}.`
                            )
                        )
                    }

                    clearTimeout(timeout)

                    const tx = Transaction.fromBuffer(response.transaction)

                    if (!tx.verify(counterparty)) return reject(Error(`Invalid transaction on channel ${chalk.yellow(channelId.toString('hex'))}.`))

                    // @TODO add some plausibility checks here

                    resolve(tx)

                    // Closes the stream
                    return false
                })
            )
        })
    }

    getAllChannels(onData, onEnd) {
        return new Promise((resolve, reject) => {
            const promises = []

            this.node.db
                .createReadStream({
                    gt: this.State(Buffer.alloc(CHANNEL_ID_LENGTH, 0x00)),
                    lt: this.State(Buffer.alloc(CHANNEL_ID_LENGTH, 0xff))
                })
                .on('error', err => reject(err))
                .on('data', ({ key, value }) => {
                    const channelId = key.slice(key.length - CHANNEL_ID_LENGTH)

                    promises.push(
                        this.state(channelId, value).then(state =>
                            onData({
                                channelId,
                                state
                            })
                        )
                    )
                })
                .on('end', () => resolve(onEnd(promises)))
        })
    }

    closeChannels() {
        return this.getAllChannels(
            channel => {
                if (channel.state.preOpened && !channel.state.lastTransaction && !channel.state.restoreTransaction) return

                return this.closeChannel(channel.channelId, channel.state)
            },
            promises => {
                if (promises.length == 0) return new BN(0)

                return Promise.all(promises).then(results => results.reduce((acc, value) => {
                    if (BN.isBN(value)) return acc.iadd(value)

                    return acc
                }, new BN(0)))
            }
        )
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

function OpenEvent(channelId) {
    return `opened ${channelId.toString('hex')}`
}

function CloseEvent(channelId) {
    return `closed ${channelId.toString('hex')}`
}

module.exports = PaymentChannel
