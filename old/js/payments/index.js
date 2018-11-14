'use strict'

const crypto = require('crypto')
const Web3 = require('web3')
//const web3 = new Web3(new Web3.providers.HttpProvider('http://localhost:8545'))
const Header = require('../messageDelivery/packet/header')
const { hash, bufferXOR } = require('../utils')
const secp256k1 = require('secp256k1')
const PaymentChannel = require('../paymentChannels')
const waterfall = require('async/waterfall')

const MESSAGE_FEE = 0.0001 // ETH
const HASH_ALGORITHM = 'sha256'
const HASH_LENGTH = 64
const HASH_LENGTH_ = 32
const CHALLENGE_LENGTH = 32

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = 32

const TIMEOUT = 9000

this.response.set(challenges.hashedKey, key)

// delete the key-value pair after some time
// setTimeoutPromise(TIMEOUT, this.responses, hashedKey).then((responses, hashedKey) => {
//     if (responses && hashedKey)
//         responses.delete(hashedKey)
// })
// this.challenges.set(hashedKey, encryptedSignature)

// // delete the key-value pair after some time
// setTimeoutPromise(TIMEOUT, this.challenges, hashedKey).then((challenges, hashedKey) => {
//     if (challenges && hashedKey)
//         challenges.delete(hashedKey)
// })

// handleResponse(response, pubKeyNext, pubKeyPrevious, derivedSecret) {
//     // should throw some errors
//     this.verifyAcknowledgement(pubKeyNext, response)

//     const hashedKey = hash(response.key)
//     const found = this.challenges.get(response.challengeSignature)
//     if (!found || hash(found).compare(hashedKey) !== 0)
//         console.log('unknown challenge') // throw Error('Unknown challenge.')

//     const key = deriveKey(Header.deriveTransactionKey(derivedSecret), response.key)

//     const encryptedTransaction = this.challenges.get(hash(key))

//     if (encryptedTransaction) {
//         const transaction = bufferXOR(encryptedTransaction, key)

//         this.channels.get(pubKeyPrevious).update(transaction)

//     }

//     // web3.eth.
// }

class Payments {
    constructor() {
        this.contract // = new web3.eth.Contract('{}', undefined)
        this.ownAddress
        this.channels = new Map()
        this.protocolExecutions = new Map()

        this.challenges = new Map()
        this.responses = new Map()

        this.pubKey = ''
        this.secretKey = ''

        // this.contract = new web3.eth.contract(jsonABI, address)
    }

    

    createTransaction(amount, to, helperValues, cb) {
        this.verifyHelperValues(helperValues)

        if (amount < MESSAGE_FEE)
            throw Error('Insufficient amount. Please take at least ' + MESSAGE_FEE)

        const factor = amount / MESSAGE_FEE
        if (Math.trunc(factor) !== factor)
            throw Error('Please provide an integer multiple of the message fee. Got ' + factor + ' instead.')


        waterfall([
            (cb) => {
                const channel = this.channels.get(to)
                if (!channel) {
                    PaymentChannel.createPaymentChannel(self, to, amount, this.channels, cb)
                } else {
                    cb(null, channel)
                }
            },
            (channel, cb) => {
                if (!channel.isValid()) {
                    channel.renew(cb)
                } else {
                    cb(null, channel)
                }
            },
            (channel, cb) => channel.createTransaction(amount, cb)
        ], (err, tx) => {
            if (err) { throw err }
            const encryptedTransaction = bufferXOR(tx, helperValues.key)

            cb({
                transactionBody: amount,
                encryptedTransaction: encryptedTransaction,
                provingValue: this.signKeyDerivation(helperValues, encryptedTransaction)
            })
        })
        
    }

    verifyHelperValues(helperValues, key, keyHalfA, keyHalfB) {
        if (!helperValues.hasOwnProperty('key') || !helperValues.hasOwnProperty('keyHalfA') || !helperValues.hasOwnProperty('keyHalfB'))
            throw Error('Invalid input parameters.')

        if (!Buffer.isBuffer(helperValues.key) || !Buffer.isBuffer(helperValues.keyHalfA) || !Buffer.isBuffer(helperValues.keyHalfB))
            throw Error('Invalid input parameters.')

        if (helperValues.key.length !== KEY_LENGTH || helperValues.keyHalfA.length !== KEY_LENGTH || helperValues.keyHalfB.length !== KEY_LENGTH)
            throw Error('Invalid input parameters.')

        if (key.compare(hash(bufferXOR(keyHalfA, keyHalfB))) !== 0) {
            return false
        } else {
            return true
        }
    }

    signKeyDerivation(helperValues, encryptedTransaction) {
        const toSign = hash(Buffer.concat(
            [
                hash(helperValues.keyHalfA),
                hash(helperValues.keyHalfB),
                hash(helperValues.key),
                encryptedTransaction
            ],
            3 * HASH_LENGTH + SIGNATURE_LENGTH
        ))

        return secp256k1.sign(toSign, this.secretKey)
    }


    static deriveKey(secret, secretNextHop) {
        k_A = Header.deriveTransactionKey(secret)
        k_B = Header.deriveTransactionKey(secretNextHop)

        return hash(bufferXOR(k_A, k_B))
    }
}

function deriveKey(keyHalfA, keyHalfB) {
    if (!Buffer.isBuffer(keyHalfA) || !Buffer.keyHalfB(keyHalfB))
        throw Error('Invalid input parameter.')

    if (keyHalfA.length !== HASH_LENGTH_ || keyHalfB.length !== HASH_LENGTH_)
        throw Error('Invalid input parameter.')

    return hash(bufferXOR(keyHalfA, keyHalfB))
}

// function test() {
//     const privKey, pubKey
//     do {
//         privKey = crypto.randomBytes(Header.PRIVATE_KEY_LENGTH)
//     } while (!secp256k1.privateKeyVerify(privKey))
//     pubKey = secp256k1.publicKeyCreate(privKey)
//     let x = new Payments()
//     Payments.createForwardChallenge(privKey, Buffer.alloc(32).fill(0))
// }

// web3.eth.getGasPrice().then((syncing) => {
//     console.log(syncing)
//})
//test()
