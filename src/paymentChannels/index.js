'use strict'

const EventEmitter = require('events');

const { CONTRACT_ADDRESS } = require('../constants')
const Web3 = require('web3')
const { parallel } = require('neo-async')
const { resolve } = require('path')
const BN = require('bn.js')
const secp256k1 = require('secp256k1')

const { pubKeyToEthereumAddress, sendTransaction, log, compileIfNecessary, isPartyA } = require('../utils')

const open = require('./rpc/open')
const closingListener = require('./eventListeners/close')
const openingListener = require('./eventListeners/open')
const transfer = require('./transfer')
const requestClose = require('./rpc/requestClose')
const closeChannels = require('./rpc/closeChannels')
const registerHandlers = require('./handlers')

const HASH_LENGTH = 32
const CHANNEL_ID_LENGTH = HASH_LENGTH
const CHALLENGE_LENGTH = 33
const PRIVATE_KEY_LENGTH = 32
const COMPRESSED_PUBLIC_KEY_LENGTH = 33

const PREFIX = Buffer.from('payments-')
const PREFIX_LENGTH = PREFIX.length

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
        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_LENGTH)
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

        if (!Buffer.isBuffer(channelId) || channelId.length !== CHANNEL_ID_LENGTH)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to opening event of channel \x1b[33m${channelId.toString('hex')}\x1b[0m`)

        this.contract.once('OpenedChannel', {
            topics: [this.web3.utils.sha3(`OpenedChannel(bytes32,uint256,uint256)`), `0x${channelId.toString('hex')}`]
        }, listener)
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
                if (!err.notFound)
                    throw err
            }

            this.node.db.createReadStream({
                gt: this.Challenge(channelId, Buffer.alloc(PRIVATE_KEY_LENGTH, 0)),
                lt: this.Challenge(channelId, Buffer.alloc(PRIVATE_KEY_LENGTH, 255))
            })
                .on('data', (obj) => {
                    const challenge = obj.key.slice(PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH, PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH + COMPRESSED_PUBLIC_KEY_LENGTH)
                    const ownKeyHalf = obj.value

                    const pubKeys = [
                        challenge,
                        secp256k1.publicKeyCreate(ownKeyHalf)
                    ]

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
     * Deletes all records that belong to a given channel.
     * 
     * @param {Buffer} channelId ID of the payment channel
     */
    deleteChannel(channelId) {
        return new Promise((resolve, reject) => {
            let batch = this.node.db.batch()
                .del(this.ChannelKey(channelId))
                .del(this.Transaction(channelId))
                .del(this.RestoreTransaction(channelId))
                .del(this.StashedRestoreTransaction(channelId))
                .del(this.Index(channelId))
                .del(this.CurrentValue(channelId))
                .del(this.InitialValue(channelId))
                .del(this.TotalBalance(channelId))

            this.node.db.createKeyStream({
                gt: this.Challenge(channelId, Buffer.alloc(COMPRESSED_PUBLIC_KEY_LENGTH, 0)),
                lt: this.Challenge(channelId, Buffer.alloc(COMPRESSED_PUBLIC_KEY_LENGTH, 255))
            })
                .on('data', (key) => {
                    console.log(key.toString())
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

        const promise = sendTransaction({
            to: this.contractAddress,
            nonce: this.nonce - 1,
            gas: estimatedGas,
            data: txObject.encodeABI()
        }, this.node.peerInfo.id, this.web3)

        if (typeof cb === 'function') {
            promise
                .then((receipt) => {
                    if (cb) {
                        cb(null, receipt)
                    } else {
                        return receipt
                    }
                })
                .catch(cb)
        } else {
            return promise
        }
    }
}

module.exports = PaymentChannel