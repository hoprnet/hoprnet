'use strict'

const secp256k1 = require('secp256k1')
const withIs = require('class-is')
const { waterfall } = require('async')

const { pubKeyToEthereumAddress, numberToBuffer, bufferToNumber, getId, isPartyA, bufferXOR } = require('../../utils')
const createKeccakHash = require('keccak')

const { toWei } = require('web3').utils

const openPaymentChannel = require('./open')

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = 32
const VALUE_LENGTH = 16
const INDEX_LENGTH = 4
const CHANNEL_ID_SIZE = 32

class Transaction {
    constructor(buf = Buffer.alloc(Transaction.SIZE), encrypted = false) {
        this.buffer = buf
        this.encrypted = encrypted
    }

    get signature() {
        return this.buffer.slice(0, SIGNATURE_LENGTH)
    }

    get value() {
        return bufferToNumber(this.buffer.slice(SIGNATURE_LENGTH + 1, SIGNATURE_LENGTH + 1 + VALUE_LENGTH))
    }

    set value(newValue) {
        this.buffer
            .slice(SIGNATURE_LENGTH + 1, SIGNATURE_LENGTH + 1 + VALUE_LENGTH)
            .fill(numberToBuffer(newValue, VALUE_LENGTH), 0, VALUE_LENGTH)
    }

    get index() {
        return bufferToNumber(this.buffer.slice(SIGNATURE_LENGTH + 1 + VALUE_LENGTH, SIGNATURE_LENGTH + 1 + VALUE_LENGTH + INDEX_LENGTH))
    }

    get recovery() {
        return this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + 1)
    }

    get channelId() {
        return this.buffer.slice(SIGNATURE_LENGTH + 1 + VALUE_LENGTH + INDEX_LENGTH, SIGNATURE_LENGTH + 1 + VALUE_LENGTH + INDEX_LENGTH + CHANNEL_ID_SIZE)
    }

    set channelId(channelId) {
        this.channelId.fill(channelId, 0, CHANNEL_ID_SIZE)
    }

    set index(newIndex) {
        this.buffer
            .slice(SIGNATURE_LENGTH + VALUE_LENGTH, SIGNATURE_LENGTH + VALUE_LENGTH + INDEX_LENGTH)
            .fill(numberToBuffer(newIndex, INDEX_LENGTH), 0, INDEX_LENGTH)
    }

    static get SIZE() {
        return SIGNATURE_LENGTH + VALUE_LENGTH + INDEX_LENGTH + 1 + CHANNEL_ID_SIZE
    }

    hash() {
        return createKeccakHash('keccak256')
            .update(this.buffer.slice(SIGNATURE_LENGTH + 1))
            .digest()
    }

    sign(node, to) {
        const signature = secp256k1.sign(
            this.hash(),
            node.peerInfo.id.privKey.marshal()
        )

        this.signature.fill(signature.signature, 0, SIGNATURE_LENGTH)
        this.recovery.fill(numberToBuffer(signature.recovery, 1), 0, 1)
    }

    verify(node, to) {
        return secp256k1.verify(this.hash(), this.signature, to.pubKey.marshal())
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length !== Transaction.SIZE)
            throw Error('Invalid input argument. Expected a buffer of size ' + Transaction.SIZE + '.')

        return new Transaction(buf)
    }

    toBuffer() {
        return this.buffer
    }

    static createTransaction(amount, to, node, cb) {
        const channelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal()))

        waterfall([
            (cb) => {
                if (node.openPaymentChannels.has(channelId.toString('base64'))) {
                    cb(null, node.openPaymentChannels.get(channelId.toString('base64')))
                } else {
                    console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Does not have a payment channel with party \'' + to.toB58String() + '\', so let\'s open one.')
                    openPaymentChannel(Transaction, to, node, cb)
                }
            },
            (lastTransaction, cb) => {
                lastTransaction.value = lastTransaction.value + getBalanceA(node, to) * amount
                lastTransaction.index = lastTransaction.index + 1

                lastTransaction.sign(node, to)

                cb(null, lastTransaction)
            }
        ], cb)
    }

    forwardTransaction(amount, to, node, cb) {
        if (amount >= this.value)
            throw Error('Node did not take the relay fee.')

        Transaction.createTransaction(amount, to, node, cb, this.buffer)
    }

    verify() {
        console.log('Received ' + this.value + ' wei.')
        return true
    }

    encrypt(key) {
        if (!Buffer.isBuffer(key) || key.length !== KEY_LENGTH)
            throw Error('Invalid key.')

        this.encrypted = true
        this.signature.fill(bufferXOR(Buffer.concat([key, key], 2 * KEY_LENGTH), this.signature), 0, SIGNATURE_LENGTH)

        return this
    }

    decrypt(key) {
        if (!Buffer.isBuffer(key) || key.length !== KEY_LENGTH)
            throw Error('Invalid key.')

        this.encrypted = false
        this.signature.fill(bufferXOR(Buffer.concat([key, key], 2 * KEY_LENGTH), this.signature), 0, SIGNATURE_LENGTH)

        return this
    }
}

function getBalanceA(node, to) {
    if (isPartyA(
        pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
        pubKeyToEthereumAddress(to.pubKey.marshal()))
    ) {
        return +1
    } else {
        return -1
    }
}

module.exports = withIs(Transaction, { className: 'Transaction', symbolName: '@validitylabs/hopper/Transaction' })