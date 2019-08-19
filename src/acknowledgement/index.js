'use strict'

const secp256k1 = require('secp256k1')

const { hash, bufferToNumber, numberToBuffer } = require('../utils')
const Header = require('../packet/header')
const { KEY_LENGTH } = require('../packet/header/parameters')

const fs = require('fs')
const protons = require('protons')

// const { Acknowledgement } = protons(fs.readFileSync(`${__dirname}/protos/acknowledgement.proto`))

const SIGNATURE_LENGTH = 64

// Format
// 32 byte key length
// 64 byte signature (challenge)
// 64 byte signature (reponse)
// 1  byte signature recovery (challenge + response)

/**
 * This class encapsulates the message that is sent back to the relayer
 * and allows that party to compute the key that is necessary to redeem
 * the previously received transaction.
 */
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
        return secp256k1.publicKeyCreate(this.key)
    }

    get hash() {
        return hash(
            Buffer.concat(
                [this.challengeSignature, numberToBuffer(this.challengeSignatureRecovery, 1), this.key], KEY_LENGTH + SIGNATURE_LENGTH))
    }

    get challengeSignature() {
        return this.buffer.slice(KEY_LENGTH, KEY_LENGTH + SIGNATURE_LENGTH)
    }

    get challengeSignatureHash() {
        return hash(Buffer.concat([this.challengeSignature, numberToBuffer(this.challengeSignatureRecovery, 1)], SIGNATURE_LENGTH + 1))
    }

    set challengeSignature(newSignature) {
        this.challengeSignature.fill(newSignature, 0, SIGNATURE_LENGTH)
    }

    get challengeSigningParty() {
        if (this._challengeSigningParty)
            return this._challengeSigningParty

        this._challengeSigningParty = secp256k1.recover(hash(this.hashedKey), this.challengeSignature, this.challengeSignatureRecovery)
        return this._challengeSigningParty
    }

    get responseSigningParty() {
        if (this._responseSigningParty)
            return this._responseSigningParty

        this._responseSigningParty = secp256k1.recover(this.hash, this.responseSignature, this.responseSignatureRecovery)
        return this._responseSigningParty
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

    sign(peerId) {
        const signature = secp256k1.sign(this.hash, peerId.privKey.marshal())

        this.responseSignature = signature.signature
        this.responseSignatureRecovery = signature.recovery
    }

    /**
     * Takes a challenge from a relayer and returns an acknowledgement that includes a
     * signature over the requested key half.
     *
     * @param {Challenge} challenge the signed challenge of the relayer
     * @param {Buffer} derivedSecret the secret that is used to create the second key half
     * @param {PeerId} peerId contains private key
     */
    static create(challenge, derivedSecret, peerId) {
        const ack = new Acknowledgement(Buffer.alloc(Acknowledgement.SIZE))

        ack.key = Header.deriveTransactionKey(derivedSecret)

        ack.challengeSignature = challenge.challengeSignature
        ack.challengeSignatureRecovery = bufferToNumber(challenge.challengeSignatureRecovery)

        ack.sign(peerId)

        return ack
    }
}

module.exports = Acknowledgement