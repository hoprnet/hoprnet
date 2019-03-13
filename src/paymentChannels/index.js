'use strict'

const EventEmitter = require('events');
const Record = require('./record')

const toPull = require('stream-to-pull-stream')
const pull = require('pull-stream')

const { CONTRACT_ADDRESS } = require('../constants')
const Web3 = require('web3')
const { parallel } = require('neo-async')
const { resolve } = require('path')
const BN = require('bn.js')

const { pubKeyToEthereumAddress, sendTransaction, log, compileIfNecessary, isPartyA } = require('../utils')

const open = require('./rpc/open')
const closingListener = require('./eventListeners/close')
const openingListener = require('./eventListeners/open')
const transfer = require('./transfer')
const requestClose = require('./rpc/requestClose')
const closeChannels = require('./rpc/closeChannels')
const registerHandlers = require('./handlers')

const HASH_LENGTH = 32
const CHANNEL_ID_BYTES = HASH_LENGTH

class PaymentChannel extends EventEmitter {
    constructor(options) {
        super()

        this.nonce = options.nonce
        this.contract = options.contract
        this.contractAddress = options.contractAddress
        this.node = options.node
        this.web3 = options.web3

        this.open = open(this)
        this.closingListener = closingListener(this)
        this.openingListener = openingListener(this)
        this.transfer = transfer(this)
        this.requestClose = requestClose(this)
        this.closeChannels = closeChannels(this)

        this.closingRequests = new Set()
        this.openingRequests = new Map()
    }

    /**
     * Creates and initializes a new PaymentChannel instance.
     * It will check whether there is an up-to-date ABI of the contract
     * and compiles the contract if that isn't the case.
     * 
     * @param {Object} options.node a libp2p node instance
     * @param {Object} options.provider a web3.js provider instance, otherwise if will use `ws://localhost:8545`
     * @param {Function} cb a function the is called with `(err, this)` afterwards
     */
    static create(options, cb) {
        const web3 = new Web3(options.provider || 'ws://localhost:8545')

        parallel({
            nonce: (cb) => web3.eth.getTransactionCount(pubKeyToEthereumAddress(options.node.peerInfo.id.pubKey.marshal()), 'latest', cb),
            compiledContract: (cb) => compileIfNecessary([resolve(__dirname, '../../contracts/HoprChannel.sol')], [resolve(__dirname, '../../build/contracts/HoprChannel.json')], cb)
        }, (err, results) => {
            if (err)
                return cb(err)

            registerHandlers(options.node)

            const abi = require('../../build/contracts/HoprChannel.json').abi

            return cb(null, new PaymentChannel({
                node: options.node,
                nonce: results.nonce,
                contract: new web3.eth.Contract(abi, options.contractAddress || CONTRACT_ADDRESS, {
                    from: pubKeyToEthereumAddress(options.node.peerInfo.id.pubKey.marshal())
                }),
                web3: web3,
                contractAddress: options.contractAddress
            }))

        })
    }

    /**
     * Registers a listener to the ClosedChannel event of a payment channel.
     * 
     * @param {Buffer} channelId ID of the channel
     * @param {Function} listener function that is called whenever the `ClosedChannel` event
     * is fired.
     */
    registerSettlementListener(channelId, listener = this.closingListener) {
        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_BYTES)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to close event of channel \x1b[33m${channelId.toString('hex')}\x1b[0m`)

        this.contract.once('ClosedChannel', {
            topics: [this.web3.utils.sha3(`ClosedChannel(bytes32,bytes16,uint256)`), `0x${channelId.toString('hex')}`]
        }, listener)
    }

    /**
     * Registers a listener to the OpenedChannel event of a payment channel.
     * 
     * @param {Buffer} channelId ID of the channel
     * @param {Function} listener function that is called whenever the `OpenedChannel` event
     * is fired.
     */
    registerOpeningListener(channelId, listener = this.openingListener) {
        if (typeof listener !== 'function')
            throw Error(`Please specify a function that is called when the close event is triggered. Got ${typeof listener} instead.`)

        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_BYTES)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to opening event of channel \x1b[33m${channelId.toString('hex')}\x1b[0m`)

        this.contract.once('OpenedChannel', {
            topics: [this.web3.utils.sha3(`OpenedChannel(bytes32,uint256,uint256)`), `0x${channelId.toString('hex')}`]
        }, listener)
    }

    /**
     * Updates the record in the database.
     * 
     * @param {Record} newRecord the new record
     * @param {object} options
     * @param {Buffer} options.channelId ID of the channel
     * @param {Object} options.sync if set to `true` it'll
     * make the changes immediately persistent
     * @param {Function} cb called when finished with `(err)`
     */
    setChannel(newRecord, options, cb) {
        if (typeof options === 'function') {
            cb = options
            options = {}
        }

        if (!newRecord.toBuffer)
            throw Error('')

        if (!options.channelId || !Buffer.isBuffer(options.channelId)) {
            if (!newRecord.restoreTx)
                return cb(Error('Unable to compute channelId.'))

            options.channelId = newRecord.restoreTx.getChannelId(this.node.peerInfo.id)
        }

        if (!options.channelId || !Buffer.isBuffer(options.channelId) || options.channelId.length !== CHANNEL_ID_BYTES)
            return cb(Error('Unable to determine channelId.'))

        const key = getKey(options.channelId)

        this.node.db.get(key, (err, record) => {
            if (err && !err.notFound)
                return cb()

            if (err && err.notFound)
                return this.node.db.put(key, newRecord.toBuffer(), cb)

            record = Record.fromBuffer(record)

            if (newRecord.tx)
                record.tx = newRecord.tx

            if (newRecord.index)
                record.index = newRecord.index

            if (newRecord.currentValue)
                record.currentValue = newRecord.currentValue

            this.node.db.put(key, record.toBuffer(), options, cb)
        })
    }

    /**
     * Fetches the local state from the database.
     * 
     * @param {Buffer} channelId ID of the channel
     * @param {Function} cb called when finished with `(err, record)`
     */
    getChannel(channelId, cb) {
        const key = getKey(channelId)

        this.node.db.get(key, (err, record) => {
            if (err)
                return cb(err.notFound ? null : err)

            cb(null, Record.fromBuffer(record))
        })
    }

    /**
     * Removes the record from the database.
     * 
     * @param {Buffer} channelId ID of the channel
     * @param {Function} cb called when finished with `(err)`
     */
    deleteChannel(channelId, cb) {
        const key = getKey(channelId)

        this.node.db.del(key, {
            sync: true
        }, cb)
    }

    /**
     * Fetches all payment channel records from the database and forward
     * them as a pull-stream to the caller.
     * 
     * @returns a pull-stream containing all stored records
     */
    getChannels() {
        return pull(
            toPull(this.node.db.createReadStream({
                // payments-channel-\000...\000
                gt: getKey(Buffer.alloc(32, 0)),
                // payments-channel-\255...\255
                lt: getKey(Buffer.alloc(46, 255))
            })),
            pull.map(record => Object.assign(record, {
                value: Record.fromBuffer(record.value)
            }))
        )
    }

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

        if (!value)
            value = '0'

        const estimatedGas = await txObject.estimateGas({
            from: pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        })

        this.nonce = this.nonce + 1

        sendTransaction({
            to: this.contractAddress,
            nonce: this.nonce - 1,
            gas: estimatedGas,
            data: txObject.encodeABI()
        }, this.node.peerInfo.id, this.web3, (err, receipt) => {
            if (cb) {
                if (err)
                    return cb(err)

                return cb(null, receipt)
            } else {
                if (err)
                    console.log(err)
            }
        })
    }
}

function getKey(channelId) {
    return Buffer.concat([Buffer.from('payments-channel-'), channelId], 17 + CHANNEL_ID_BYTES)
}

module.exports = PaymentChannel