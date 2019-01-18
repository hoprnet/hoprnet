'use strict'

const { sign, verify, publicKeyVerify, recover } = require('secp256k1')
const { parallel } = require('async')

const { hash, bufferToNumber, numberToBuffer } = require('./utils')
const Header = require('./packet/header')
const { KEY_LENGTH } = require('./packet/header/parameters')

const SIGNATURE_LENGTH = 64

// Format
// 32 byte key length
// 64 byte signature (challenge)
// 64 byte signature (reponse)
// 1  byte signature recovery (challenge + response)

class Acknowledgement {
    constructor(buf) {
        this.buffer = buf
    }

    get key() {
        return this.buffer.slice(0, KEY_LENGTH)
    }

    set key(newKey) {
        this.key.fill(newKey, 0, KEY_LENGTH)
    }

    get hashedKey() {
        return hash(this.key)
    }

    get hash() {
        return hash(
            Buffer.concat(
                [this.key, this.challengeSignature], KEY_LENGTH + SIGNATURE_LENGTH))
    }

    get challengeSignature() {
        return this.buffer.slice(KEY_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH)
    }

    set challengeSignature(newSignature) {
        this.challengeSignature.fill(newSignature, 0, SIGNATURE_LENGTH)
    }

    get challengeSigningParty() {
        return recover(this.hashedKey, this.challengeSignature, this.challengeSignatureRecovery)
    }

    get responseSigningParty() {
        return recover(this.hash, this.responseSignature, this.responseSignatureRecovery)
    }

    get challengeSignatureRecovery() {
        return bufferToNumber(this.buffer
            .slice(KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH + 1)) >> 1
    }

    get responseSignatureRecovery() {
        return bufferToNumber(this.buffer
            .slice(KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH + 1)) % 2
    }

    set challengeSignatureRecovery(newRecovery) {
        if (newRecovery > 1)
            throw Error(`Invalid recovery value. Expected either 0 or 1, got ${newRecovery}.`)

        const recovery = this.buffer
            .slice(KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH + 1)

        // save last bit
        const tmp = bufferToNumber(recovery) % 2

        // shift right to clear both bits, then add first bit and shifted second bit
        recovery.fill(numberToBuffer((bufferToNumber(recovery) >> 2) + tmp + (newRecovery << 1), 1), 0, 1)
    }

    set responseSignatureRecovery(newRecovery) {
        if (newRecovery > 1)
            throw Error(`Invalid recovery value. Expected either 0 or 1, got ${newRecovery}.`)

        const recovery = this.buffer
            .slice(KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH + 1)

        // Shift right, then left to clear last bit, then set new recovery value
        // by adding it to the result.
        recovery.fill(numberToBuffer(((bufferToNumber(recovery) >> 1) << 1) + newRecovery, 1), 0, 1)
    }

    get responseSignature() {
        return this.buffer.slice(KEY_LENGTH + SIGNATURE_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH)
    }

    set responseSignature(newSignature) {
        this.responseSignature.fill(newSignature, 0, SIGNATURE_LENGTH)
    }

    static get SIZE() {
        return KEY_LENGTH + SIGNATURE_LENGTH + SIGNATURE_LENGTH + 1
    }

    toBuffer() {
        return this.buffer
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error(`Invalid input. Expected a buffer. Got '${typeof buf}'.`)

        if (!Buffer.isBuffer(buf) || buf.length !== this.SIZE)
            throw Error(`Expected a buffer of size ${this.SIZE}. Got a buffer of size ${buf.length}.`)

        return new Acknowledgement(buf)
    }

    signResponse(peerId) {
        const signature = sign(this.hash, peerId.privKey.marshal())

        this.responseSignature = signature.signature
        this.responseSignatureRecovery = signature.recovery
    }

    /**
     * 
     * @param {*} challenge 
     * @param {*} derivedSecret 
     * @param {PeerId} peerId contains private key 
     * @param {Buffer} buffer (optional) specify a buffer which is used to create the instance
     */
    static create(challenge, derivedSecret, peerId, buffer = Buffer.alloc(Acknowledgement.SIZE)) {
        const ack = new Acknowledgement(buffer)

        ack.key = Header.deriveTransactionKey(derivedSecret)

        ack.challengeSignature = challenge.challengeSignature
        ack.challengeSignatureRecovery = bufferToNumber(challenge.challengeSignatureRecovery)

        ack.signResponse(peerId)

        return ack
    }

    verify(pubKeyNext, ownPubkey, cb) {
        if (!Buffer.isBuffer(pubKeyNext) || !publicKeyVerify(pubKeyNext))
            throw Error('Invalid public key.')

        parallel([
            (cb) => cb(null, verify(hash(this.key), this.challengeSignature, ownPubkey)),
            (cb) => cb(null, verify(this.hash, this.responseSignature, pubKeyNext))
        ], (err, results) => cb(err, results.every(x => x)))
    }
}

module.exports = Acknowledgement