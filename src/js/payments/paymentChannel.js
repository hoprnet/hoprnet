'use strict'

const secp256k1 = require('secp256k1')
const utils = require('web3').utils

class PaymentChannel {
    constructor(pubKeyA, amountA, pubKeyB, amountB) {
        this.pubKeyA = pubKeyA
        this.amountA = amountA

        this.pubKeyB = pubKeyB
        this.amountB = amountB
    }

    transfer(secretKey, amount, from, to) {
        if (amount <= 0)
            throw Error('Invalid amount.')

        if (!Buffer.isBuffer(from) || !Buffer.isBuffer(to) || !Buffer.isBuffer(secretKey))
            throw Error('Invalid arguments.')
            
        if (from.compare(to) === 0)
            throw Error('Source and destination are equal.')

        if (!secp256k1.publicKeyVerify(from) || !secp256k1.publicKeyVerify(to) || !secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid keys')

        if (from.compare(this.pubKeyA) === 0 && to.compare(this.pubKeyB) === 0) {
            if (this.amountA < amount)
                throw Error('Insufficient funds')

            this.amountA = this.amountA - amount
            this.amountB = this.amountB + amount

        } else if (from.compare(this.pubKeyB) === 0 && to.compare(this.pubKeyA) === 0) {
            if (this.amountB < amount)
                throw Error('Insufficient funds')


            this.amountB = this.amountB - amount
            this.amountA = this.amountA + amount

        } else {
            throw Error('General error.')
        }

        let bitstr = utils.sha3(
            Buffer.from(amount.toString() + ' ' + from + ' ' + to)
        ).slice(2)
        
        return secp256k1.sign(Buffer.from(bitstr), secretKey)
    }
}