'use strict'

const crypto = require('crypto')
const Web3 = require('web3')
//const web3 = new Web3(new Web3.providers.HttpProvider('http://localhost:8545'))
const Header = require('../messageDelivery/header')
const { bufferXOR_in_place, bufferXOR } = require('../utils')
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


let ack = {
    pubKey: '',
}

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

    createChallenge(hashedKey, encryptedSignature) {
        if (!Buffer.isBuffer(hashedKey) || hashedKey.length !== HASH_LENGTH_)
            throw Error('Invalid hashed key format.')

        let signedChallenge = secp256k1.sign(hashedKey, this.secretKey)

        this.challenges.set(hashedKey, encryptedSignature)

        // delete the key-value pair after some time
        // setTimeoutPromise(TIMEOUT, this.challenges, hashedKey).then((challenges, hashedKey) => {
        //     if (challenges && hashedKey)
        //         challenges.delete(hashedKey)
        // })

        return {
            hashedKey: hashedKey,
            signature: signedChallenge
        }
    }

    verifyChallenge(pubKey, challenge) {
        if (!challenge || !challenge.hasOwnProperty('signature') || !challenge.hasOwnProperty('hashedKey'))
            throw Error('Invalid challenge format.')

        if (!Buffer.isBuffer(pubKey) || !secp256k1.publicKeyVerify(pubKey))
            throw Error('Invalid public key.')

        if (!Buffer.isBuffer(challenge.signature) || challenge.signature.length !== SIGNATURE_LENGTH)
            throw Error('Invalid signature format.')

        if (!Buffer.isBuffer(challenge.hashedKey) || !challenge.hashedKey.length != CHALLENGE_LENGTH)
            throw Error('Invalid challenge format.')

        if (!secp256k1.verify(challenge.hashedKey, challenge.signature, pubKey)) {
            throw Error('General error.')
        }
    }

    createResponse(pubKeyPrevious, challenge, derivedSecret) {
        // should throw some errors if anything is wrong
        this.verifyChallenge(pubKeyPrevious, challenge)

        let key = Header.deriveTransactionKey(derivedSecret)

        let toSign = hash(
            Buffer.concat(
                [
                    key,
                    challenge.signature
                ], HASH_LENGTH_ + SIGNATURE_LENGTH)
        )

        let response = {
            key: key,
            challengeSignature: challenge.signature,
            signature: secp256k1.sign(toSign, this.secretKey)
        }

        this.response.set(challenges.hashedKey, key)

        // delete the key-value pair after some time
        setTimeoutPromise(TIMEOUT, this.responses, hashedKey).then((responses, hashedKey) => {
            if (responses && hashedKey)
                responses.delete(hashedKey)
        })

        return response
    }

    verifyResponse(pubKeyNext, response) {
        if (!Buffer.isBuffer(pubKeyNext) || !secp256k1.publicKeyVerify(pubKeyNext))
            throw Error('Invalid public key.')

        if (!response || !response.hasOwnProperty('key') || !response.hasOwnProperty('signature') || !response.hasOwnProperty('challengeSignature'))
            throw Error('Invalid response format.')

        if (!Buffer.isBuffer(response.key) || !Buffer.isBuffer(response.signature) || !Buffer.isBuffer(response.challengeSignature))
            throw Error('Invalid response format.')

        if (response.key.length !== HASH_LENGTH_ || response.signature.length !== SIGNATURE_LENGTH || response.challengeSignature.length !== SIGNATURE_LENGTH)
            throw Error('Invalid response format.')

        let hashedKey = hash(challenges.key)

        if (!secp256k1.verify(hashedKey, response.challengeSignature, this.pubKey))
            throw Error('General error.')

        let toVerify = hash(
            Buffer.concat(
                [
                    key,
                    response.challengeSignature
                ], HASH_LENGTH_ + SIGNATURE_LENGTH)
        )

        if (!secp256k1.verify(toVerify, response.signature, pubKeyNext))
            throw Error('General error.')
    }

    handleResponse(response, pubKeyNext, pubKeyPrevious, derivedSecret) {
        // should throw some errors
        this.verifyResponse(pubKeyNext, response)

        let hashedKey = hash(response.key)
        let found = this.challenges.get(response.challengeSignature)
        if (!found || hash(found).compare(hashedKey) !== 0)
            console.log('unknown challenge') // throw Error('Unknown challenge.')

        let key = deriveKey(Header.deriveTransactionKey(derivedSecret), response.key)

        let encryptedTransaction = this.challenges.get(hash(key))

        if (encryptedTransaction) {
            let transaction = bufferXOR(encryptedTransaction, key)

            this.channels.get(pubKeyPrevious).update(transaction)

        }

        // web3.eth.
    }

    createTransaction(amount, to, helperValues, cb) {
        this.verifyHelperValues(helperValues)

        if (amount < MESSAGE_FEE)
            throw Error('Insufficient amount. Please take at least ' + MESSAGE_FEE)

        let factor = amount / MESSAGE_FEE
        if (Math.trunc(factor) !== factor)
            throw Error('Please provide an integer multiple of the message fee. Got ' + factor + ' instead.')


        waterfall([
            (cb) => {
                let channel = this.channels.get(to)
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
            let encryptedTransaction = bufferXOR(tx, helperValues.key)

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
        let toSign = hash(Buffer.concat(
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

function hash(buf) {
    if (!Buffer.isBuffer(buf))
        throw Error('Invalid input. Please use a Buffer')

    return Buffer.from(
        web3.utils.sha3(buf).slice(2),
        'hex'
    )
}

function deriveKey(keyHalfA, keyHalfB) {
    if (!Buffer.isBuffer(keyHalfA) || !Buffer.keyHalfB(keyHalfB))
        throw Error('Invalid input parameter.')

    if (keyHalfA.length !== HASH_LENGTH_ || keyHalfB.length !== HASH_LENGTH_)
        throw Error('Invalid input parameter.')

    return hash(bufferXOR(keyHalfA, keyHalfB))
}

function test() {
    let privKey, pubKey
    do {
        privKey = crypto.randomBytes(Header.PRIVATE_KEY_LENGTH)
    } while (!secp256k1.privateKeyVerify(privKey))
    pubKey = secp256k1.publicKeyCreate(privKey)
    let x = new Payments()
    Payments.createForwardChallenge(privKey, Buffer.alloc(32).fill(0))
}

// web3.eth.getGasPrice().then((syncing) => {
//     console.log(syncing)
//})
//test()
