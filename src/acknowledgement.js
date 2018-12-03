'use strict'

const withIs = require('class-is')
const secp256k1 = require('secp256k1')
const parallel = require('async/parallel')

const { hash } = require('./utils')
const Header = require('./packet/header')
const p = require('./packet/header/parameters')

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = p.KEY_LENGTH

class Acknowledgement {
    constructor(buf) {
        this.buffer = buf
    }

    get key() {
        return this.buffer.slice(0, KEY_LENGTH)
    }

    get hashedKey() {
        return hash(this.key)
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

    hash() {
        return hash(
            Buffer.concat(
                [this.key, this.challengeSignature], KEY_LENGTH + SIGNATURE_LENGTH))
    }

    static create(challenge, derivedSecret, secretKey, buffer = Buffer.alloc(Acknowledgement.SIZE)) {
        const ack = new Acknowledgement(buffer)

        ack.key
            .fill(Header.deriveTransactionKey(derivedSecret), 0, KEY_LENGTH)

        ack.challengeSignature
            .fill(challenge.challengeSignature, 0, SIGNATURE_LENGTH)

        // console.log('challengeSignature ' + ack.challengeSignature.toString('base64'))

        ack.responseSignature
            .fill(
                secp256k1.sign(
                    ack.hash(),
                    secretKey).signature,
                0, SIGNATURE_LENGTH)

        return ack
    }

    verify(pubKeyNext, ownPubkey, cb) {
        if (!Buffer.isBuffer(pubKeyNext) || !secp256k1.publicKeyVerify(pubKeyNext))
            throw Error('Invalid public key.')

        parallel([
            (cb) => cb(null, secp256k1.verify(hash(this.key), this.challengeSignature, ownPubkey)),
            (cb) => cb(null, secp256k1.verify(this.hash(), this.responseSignature, pubKeyNext))
        ], (err, results) => cb(err, results.every(x => x)))
    }
}

module.exports = withIs(Acknowledgement, { className: 'Acknowledgement', symbolName: '@validitylabs/hopper/Acknowledgement' })