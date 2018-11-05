'use strict'

const withIs = require('class-is')
const secp256k1 = require('secp256k1')

const { hash } = require('../utils')
const Header = require('../messageDelivery/packet/header')

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = SIGNATURE_LENGTH

class Acknowledgement {
    constructor(buf) {
        this.buffer = buf
    }

    get key() {
        return this.buffer.slice(0, KEY_LENGTH)
    }

    get challengeSignature() {
        return this.buffer.slice(KEY_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH)
    }

    get responseSignature() {
        return this.buffer.slice(KEY_LENGTH + SIGNATURE_LENGTH, KEY_LENGTH + 2 * SIGNATURE_LENGTH)
    }

    static get SIZE() {
        return KEY_LENGTH + 2 * SIGNATURE_LENGTH
    }

    toBuffer() {
        return this.buffer
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Invalid input. Expected a buffer. Got \"' + typeof buf + '\".')

        if (!Buffer.isBuffer(buf) || buf.length !== KEY_LENGTH + 2 * SIGNATURE_LENGTH)
            throw Error('Expected a buffer of size ' + KEY_LENGTH + 2 * SIGNATURE_LENGTH + '. Got a buffer of size ' + buf.length + '.')

        return new Acknowledgement(buf)
    }

    static create(challenge, derivedSecret, secretKey) {
        const key = Header.deriveTransactionKeyBlinding(derivedSecret)

        const buf = Buffer.concat(
            [key, challenge.challengeSignature, Buffer.alloc(SIGNATURE_LENGTH)],
            KEY_LENGTH + 2 * SIGNATURE_LENGTH)

        Buffer.from(secp256k1.sign(hash(buf.slice(0, KEY_LENGTH + SIGNATURE_LENGTH)), secretKey).signature)
            .copy(buf, KEY_LENGTH + SIGNATURE_LENGTH, 0, SIGNATURE_LENGTH)

        return new Acknowledgement(buf)
    }

    isValid(pubKeyNext) {
        if (!Buffer.isBuffer(pubKeyNext) || !secp256k1.publicKeyVerify(pubKeyNext))
            throw Error('Invalid public key.')

        const hashedKey = hash(this.key)

        if (!secp256k1.verify(hashedKey, this.challengeSignature, pubKeyNext))
            return false

        if (!secp256k1.verify(this.buffer.slice(0, KEY_LENGTH + SIGNATURE_LENGTH), response.signature, pubKeyNext))
            return false

        return true
    }
}

module.exports = withIs(Acknowledgement, { className: 'Acknowledgement', symbolName: '@validitylabs/hopper/Acknowledgement' })