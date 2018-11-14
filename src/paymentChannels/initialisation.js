'use strict'

const secp256k1 = require('secp256k1')

const { numberToBuffer, bufferToNumber } = require('../utils')

class Transaction {
    constructor(buf, encrypted = false) {
        this.buffer = buf
        this.encrypted = encrypted
    }

    get signature() {
        return this.buffer.slice(0, SIGNATURE_LENGTH)
    }

    get value() {
        return bufferToNumber(this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + VALUE_LENGTH))
    }

    static get SIZE() {
        return SIGNATURE_LENGTH + VALUE_LENGTH
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length !== Transaction.SIZE)
            throw Error('Invalid input argument. Expected a buffer of size ' + Transaction.SIZE + '.')
        return new Transaction(buf)
    }

    toBuffer() {
        return this.buffer
    }

    static createTransaction(amount, to, secretKey, buffer = Buffer.alloc(Transaction.SIZE)) {
        // if (!Buffer.isBuffer(to) || !secp256k1.publicKeyVerify(to))
        //     throw Error('Invalid recipient.')

        return new Transaction(buffer
            .fill(0, 0, SIGNATURE_LENGTH)
            .fill(numberToBuffer(amount, VALUE_LENGTH), SIGNATURE_LENGTH, SIGNATURE_LENGTH + VALUE_LENGTH))
    }

    forwardTransaction(amount, to, secretKey) {
        if (amount >= this.value)
            throw Error('Node did not take the relay fee.')
            
        Transaction.createTransaction(amount, to, secretKey, this.buffer)
    }

    verify() {
        console.log('Received ' + this.value + ' coins.')
        return true
    }

    encrypt(key) {
        if (!Buffer.isBuffer(key) || key.length !== KEY_LENGTH)
            throw Error('Invalid key.')

        this.encrypted = true
        this.signature = bufferXOR(key, this.signature)
    }

    decrypt(key) {
        if (!Buffer.isBuffer(key) || key.length !== KEY_LENGTH)
            throw Error('Invalid key.')

        this.encrypted = false
        this.signature = bufferXOR(key, this.signature)
    }
}

module.exports = withIs(Transaction, { className: 'Transaction', symbolName: '@validitylabs/hopper/Transaction' })