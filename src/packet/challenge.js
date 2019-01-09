'use strict'

const { sign, privateKeyVerify, publicKeyVerify, verify } = require('secp256k1')

const { hash } = require('../utils')

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = 32
const HASH_LENGTH = 32
const HASH_KEY_KEY_HALF = 'KEY_HALF'

class Challenge {
    constructor(buf) {
        this.buffer = buf
    }

    get challengeSignature() {
        return this.buffer.slice(0, SIGNATURE_LENGTH)
    }

    static get SIZE() {
        return SIGNATURE_LENGTH
    }

    toBuffer() {
        return this.buffer
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Invalid input. Expected a buffer. Got \"' + typeof buf + '\".')

        if (buf.length !== SIGNATURE_LENGTH)
            throw Error('Expected a buffer of size ' + SIGNATURE_LENGTH + '. Got a buffer of size ' + buf.length + '.')

        return new Challenge(buf)
    }

    static createChallenge(secret, secretKey, buffer = Buffer.alloc(SIGNATURE_LENGTH)) {
        // console.log('Create challenge with secret ' + secret.toString('base64'))
        if (!Buffer.isBuffer(secret))
            throw Error('Invalid secret format.')

        if (!privateKeyVerify(secretKey))
            throw Error('Invalid private key format.')

        const challenge = new Challenge(buffer)

        challenge.challengeSignature
            .fill(sign(Challenge.deriveHashedKey(secret), secretKey).signature, 0, SIGNATURE_LENGTH)
        // console.log('create challenge with signature' + challenge.challengeSignature.toString('base64'))

        return challenge
    }

    updateChallenge(hashedKey, secretKey, cb) {
        if (!Buffer.isBuffer(hashedKey) || hashedKey.length !== HASH_LENGTH)
            throw Error('Wrong input value. Expected a hashed key of size ' + HASH_LENGTH + ' bytes.')

        if (!privateKeyVerify(secretKey))
            throw Error('Invalid private key format.')

        this.challengeSignature
            .fill(sign(hashedKey, secretKey).signature, 0, SIGNATURE_LENGTH)
        
        cb(null, this)
    }

    static deriveHashedKey(secret) {
        return hash(secret)
    }

    verify(pubKey, secret) {
        if (!Buffer.isBuffer(pubKey) || !publicKeyVerify(pubKey))
            throw Error('Invalid public key.')

        console.log(verify(Challenge.deriveHashedKey(secret), this.challengeSignature, pubKey) ? 'Verification OK.' : 'Verification failed.')
        return verify(Challenge.deriveHashedKey(secret), this.challengeSignature, pubKey)
    }
}

module.exports = Challenge