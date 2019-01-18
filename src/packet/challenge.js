'use strict'

const { sign, verify } = require('secp256k1')

const { hash, numberToBuffer } = require('../utils')

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

    get challengeSignatureRecovery() {
        return this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + 1)
    }

    static get SIZE() {
        return SIGNATURE_LENGTH + 1
    }

    toBuffer() {
        return this.buffer
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error(`Invalid input. Expected a buffer. Got '${typeof buf}' instead.`)

        if (buf.length !== Challenge.SIZE)
            throw Error(`Expected a buffer of size ${Challenge.SIZE}. Got a buffer of size ${buf.length}.`)

        return new Challenge(buf)
    }

    signChallenge(hashedKey, peerId) {
        const signature = sign(hashedKey, peerId.privKey.marshal())

        this.challengeSignature
            .fill(signature.signature, 0, SIGNATURE_LENGTH)

        this.challengeSignatureRecovery
            .fill(numberToBuffer(signature.recovery, 1), 0, 1)
    }

    /**
     * 
     * @param {Buffer} secret 
     * @param {PeerId} peerId contains secret key
     * @param {Buffer} buffer (optional) Buffer to store the generated Challenge instance
     */
    static createChallenge(secret, peerId, buffer = Buffer.alloc(Challenge.SIZE)) {
        if (!Buffer.isBuffer(secret))
            throw Error('Invalid secret format.')

        const challenge = new Challenge(buffer)
        
        challenge.signChallenge(Challenge.deriveHashedKey(secret), peerId)

        return challenge
    }

    /**
     * 
     * @param {*} hashedKey 
     * @param {*} peerId contains the secret key
     * @param {*} cb 
     */
    updateChallenge(hashedKey, peerId, cb) {
        if (!Buffer.isBuffer(hashedKey) || hashedKey.length !== HASH_LENGTH)
            throw Error(`Wrong input value. Expected a hashed key of size ${HASH_LENGTH} bytes.`)

        this.signChallenge(hashedKey, peerId)
        
        cb(null, this)
    }

    static deriveHashedKey(secret) {
        return hash(secret)
    }

    verify(peerId, secret) {
        if (!peerId.pubKey)
            throw Error('Unable to verify challenge without a public key.')

        return verify(Challenge.deriveHashedKey(secret), this.challengeSignature, peerId.pubKey.marshal())
    }
}

module.exports = Challenge