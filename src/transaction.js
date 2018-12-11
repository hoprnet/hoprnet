'use strict'

const secp256k1 = require('secp256k1')
const withIs = require('class-is')

const { hash, numberToBuffer, bufferToNumber, bufferXOR, getId, pubKeyToEthereumAddress } = require('./utils')

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = 32
const VALUE_LENGTH = 32
const INDEX_LENGTH = 32
const CHANNEL_ID_SIZE = 32

class Transaction {
    constructor(buf = Buffer.alloc(Transaction.SIZE), encrypted = false) {
        this.buffer = buf
        this.encrypted = encrypted
    }

    get signature() {
        return this.buffer.slice(0, SIGNATURE_LENGTH)
    }

    get recovery() {
        return this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + 1)
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

    set index(newIndex) {
        this.buffer
            .slice(SIGNATURE_LENGTH + 1 + VALUE_LENGTH, SIGNATURE_LENGTH + 1 + VALUE_LENGTH + INDEX_LENGTH)
            .fill(numberToBuffer(newIndex, INDEX_LENGTH), 0, INDEX_LENGTH)
    }

    get channelId() {
        return this.buffer.slice(SIGNATURE_LENGTH + 1 + VALUE_LENGTH + INDEX_LENGTH, SIGNATURE_LENGTH + 1 + VALUE_LENGTH + INDEX_LENGTH + CHANNEL_ID_SIZE)
    }

    set channelId(channelId) {
        this.channelId.fill(channelId, 0, CHANNEL_ID_SIZE)
    }

    static get SIZE() {
        return SIGNATURE_LENGTH + VALUE_LENGTH + INDEX_LENGTH + 1 + CHANNEL_ID_SIZE
    }

    hash() {
        return hash(this.buffer.slice(SIGNATURE_LENGTH + 1))
    }

    sign(privKey) {
        const signature = secp256k1.sign(this.hash(), privKey)

        this.signature.fill(signature.signature, 0, SIGNATURE_LENGTH)
        this.recovery.fill(numberToBuffer(signature.recovery, 1), 0, 1)
    }

    verify(node) {
        return getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(secp256k1.recover(this.hash(), this.signature, bufferToNumber(this.recovery)))
        ).compare(this.channelId) === 0
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length !== Transaction.SIZE)
            throw Error('Invalid input argument. Expected a buffer of size ' + Transaction.SIZE + '.')

        return new Transaction(buf)
    }

    toBuffer() {
        return this.buffer
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

module.exports = withIs(Transaction, { className: 'Transaction', symbolName: '@validitylabs/hopper/Transaction' })