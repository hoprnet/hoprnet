'use strict'

const crypto = require('crypto')
const web3 = require('web3')
const Header = require('../messageDelivery/header')
const { bufferXOR_in_place, bufferXOR } = require('../utils')
const secp256k1 = require('secp256k1')
const PaymentChannel = require('./paymentChannel')

const MESSAGE_FEE = 0.0001 // ETH
const HASH_ALGORITHM = 'keccak256'
const HASH_LENGTH = 64

const SIGNATURE_LENGTH = 64
// create payment channel

// close payment channel

// update payment channel

class Payments {
    constructor(decrypted, ) {
        //this.channel = new PaymentChannel()

        this.amountA = null
        this.amountB = null
    }

    static createProvingValues(secretKey, hop, nextHop) {
        let keyHalfA = web3.utils.sha3(
            bufferXOR_in_place(
                Header.deriveTransactionKey(hop),
                Header.deriveTransactionKeyBlinded(hop)
            )
        )
        let keyHalfB = web3.utils.sha3(
            bufferXOR_in_place(
                Header.deriveTransactionKey(nextHop),
                Header.deriveTransactionKeyBlinded(hop)
            )
        )
        let key = bufferXOR(
            keyHalfA,
            keyHalfB
        )
        let hashedKey = web3.utils.sha3(key)

        let encryptedSignature = bufferXOR(
            key,
            this.channel.transfer(secretKey, MESSAGE_FEE, hop, nextHop)
        )

        return secp256k1.sign(
            web3.utils.sha3(
                Buffer.concat(
                    [
                        keyHalfA,
                        keyHalfB,
                        hashedKey,
                        encryptedSignature
                    ]
                )
            ).slice(2),
            secretKey
        )
    }

    static createForwardChallenge(secretKey, hashedKey) {
        if (!Buffer.isBuffer(secretKey) || !secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid secret key.')

        if (!Buffer.isBuffer(hashedKey))
            throw Error('Invalid key.')

        return secp256k1.sign(
            Buffer.from(web3.utils.sha3(hashedKey).slice(2), 'hex'),
            secretKey
        )
    }

    static createAcknowledgement(publicKeySender, derivedSecret, secretKey, signature) {
        if (!Buffer.isBuffer(publicKeySender) || !secp256k1.publicKeyVerify(publicKeySender))
            throw Error('Invalid secret key.')

        if (!Buffer.isBuffer(secretKey) || !secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid secret key.')

        if (!Buffer.isBuffer(signature) || signature.length != SIGNATURE_LENGTH)
            throw Error('Invalid signature format.')

        let key = Header.deriveTransactionKey(derivedSecret)

        if (!secp256k1.verify(
            web3.utils.sha3(key).slice(2),
            signature,
            publicKeySender
        ))
            throw Error('General error.') // TODO: Call BullShitContract

        return secp256k1.sign(
            Buffer.from(
                web3.utils.sha3(
                    Buffer.concat([signature, key], HASH_LENGTH + SIGNATURE_LENGTH)
                ).slice(2),
                'hex'
            ),
            secretKey
        )
    }

    static verifyAcknowledgement(signatureNextHop, key, nextPubKey, signature, hashedKey, pubkey) {
        if (!Buffer.isBuffer(signatureNextHop) || !Buffer.isBuffer(key) || !Buffer.isBuffer(signature) || !Buffer.isBuffer(hashedKey))
            throw Error('Invalid input values.')

        if (signatureNextHop.length !== SIGNATURE_LENGTH || signature.length !== SIGNATURE_LENGTH)
            throw Error('Invalid signature format.')

        if (key.length !== HASH_LENGTH || hashedKey.length !== HASH_LENGTH)
            throw Error('Invalid input format.')

        if (Buffer.from(web3.utils.sha3(key).slice(2), 'hex').compare(hashedKey) !== 0)
            throw Error('General error.')

        if (!secp256k1.verify(hashedKey, signature, pubkey))
            throw Error('General error.')

        let bitStr = Buffer.from(
            web3.utils.sha3(
                Buffer.concat([signature, key], HASH_LENGTH + SIGNATURE_LENGTH)
            ), 'hex')

        if (!secp256k1.verify(bitStr, signatureNextHop, nextPubKey))
            throw Error('General error.')

        return key
    }

    static deriveKey(secret, secretNextHop) {
        k_A = Header.deriveTransactionKey(secret)
        k_B = Header.deriveTransactionKey(secretNextHop)

        return Buffer.from(
            web3.utils.sha3(bufferXOR(k_A, k_B)).slice(2), 'hex'
        )
    }

    static createHelperValues(secret, secretNextHop, secretNextNextHop) {
        if (!Buffer.isBuffer(secret) || !Buffer.isBuffer(secretNextHop) || !Buffer.isBuffer(secretNextNextHop))
            throw Error('Invalid input values.')

        if (!secp256k1.publicKeyVerify(secret) || !secp256k1.publicKeyVerify(secretNextHop) || !secp256k1.publicKeyVerify(secretNextNextHop))
            throw Error('Invalid keys.')

        let k_A_blinder = Header.deriveTransactionKeyBlinded(secret)
        let k_B = Header.deriveTransactionKeyBlinded(secretNextHop)

        let hashedKeyHalfB = Buffer.from(web3.utils.sha3(bufferXOR(k_B, k_A_blinder)).slice(2), 'hex')

        let key_AB = Payments.deriveKey(secret, secretNextHop)
        let key_C = Header.deriveTransactionKey(secretNextNextHop)

        let hashed_key_AB = Buffer.from(web3.utils.sha3(key_AB).slice(2), 'hex')
        let hashed_key_C = Buffer.from(web3.utils.sha3(key_C).slice(2), 'hex')

        return Buffer.concat([
            hashed_key_AB,
            hashed_key_C,
            Payments.deriveKey(secretNextHop, secretNextNextHop),
            hashedKeyHalfB,
        ], 4 * HASH_LENGTH)
    }
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
test()
