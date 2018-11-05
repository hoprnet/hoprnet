'use strict'

const secp256k1 = require('secp256k1')
const withIs = require('class-is')
const hkdf = require('futoin-hkdf')


const SIGNATURE_LENGTH = 64
const KEY_LENGTH = 32
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
        console.log('Create challenge with secret ' + secret.toString('base64'))
        if (!Buffer.isBuffer(secret))
            throw Error('Invalid secret format.')

        if (!secp256k1.privateKeyVerify(secretKey))
            throw Error('Invalid private key format.')

        const hashedKey = Challenge.deriveHashedKey(secret)

        buffer.fill(secp256k1.sign(hashedKey, secretKey).signature, 0, SIGNATURE_LENGTH)

        return new Challenge(buffer)
    }

    updateChallenge(secret, secretKey) {
        Challenge.createChallenge(secret, secretKey, this.buffer)
    }

    static deriveHashedKey(secret) {
        return hkdf(secret, KEY_LENGTH, { salt: HASH_KEY_KEY_HALF })
    }

    verify(pubKey, secret) {
        console.log('Verify challenge with secret ' + secret.toString('base64'))
        if (!Buffer.isBuffer(pubKey) || !secp256k1.publicKeyVerify(pubKey))
            throw Error('Invalid public key.')

        return secp256k1.verify(Challenge.deriveHashedKey(secret), this.challengeSignature, pubKey)
    }
}

module.exports = withIs(Challenge, { className: 'Challenge', symbolName: '@validitylabs/hopper/Challenge' })