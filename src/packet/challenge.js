'use strict'

const secp256k1 = require('secp256k1')

const { hash, numberToBuffer, bufferToNumber } = require('../utils')

const SIGNATURE_LENGTH = 64
const COMPRESSED_PUBLIC_KEY_LENGTH = 33

/**
 * The purpose of this class is to give the relayer the opportunity to claim
 * the proposed funds in case the the next downstream node responds with an 
 * inappropriate acknowledgement.
 */
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

    get signatureHash() {
        return hash(this.buffer.slice(0, SIGNATURE_LENGTH + 1))
    }

    /**
     * Uses the derived secret and the signature to recover the public
     * key of the signer.
     */
    get counterparty() {
        if (this._counterparty)
            return this._counterparty

        if (!this._hashedKey)
            throw Error(`Can't recover public key without challenge.`)

        this._counterparty = secp256k1.recover(hash(this._hashedKey), this.challengeSignature, bufferToNumber(this.challengeSignatureRecovery))
        return this._counterparty
    }

    static get SIZE() {
        return SIGNATURE_LENGTH + 1
    }

    toBuffer() {
        return this.buffer
    }

    /**
     * Recovers a challenge from a buffer. Used to deserialize the challenge object.
     * 
     * @param {Buffer} buf that contains a challenge
     */
    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error(`Invalid input. Expected a buffer. Got '${typeof buf}' instead.`)

        if (buf.length !== Challenge.SIZE)
            throw Error(`Expected a buffer of size ${Challenge.SIZE}. Got a buffer of size ${buf.length}.`)

        return new Challenge(buf)
    }

    /**
     * Signs the challenge and includes the transferred amount of money as
     * well as the ethereum address of the signer into the signature.
     * 
     * @param {PeerId} peerId that contains private key and public key of the node
     */
    sign(peerId) {
        // const hashedChallenge = hash(Buffer.concat([this._hashedKey, this._fee.toBuffer('be', VALUE_LENGTH)], HASH_LENGTH + VALUE_LENGTH))
        const signature = secp256k1.sign(hash(this._hashedKey), peerId.privKey.marshal())

        this.challengeSignature
            .fill(signature.signature, 0, SIGNATURE_LENGTH)

        this.challengeSignatureRecovery
            .fill(numberToBuffer(signature.recovery, 1), 0, 1)

        return this
    }

    /**
     * Creates a challenge object.
     * 
     * @param {Buffer} hashedKey that is used to generate the key half
     * @param {BN} fee 
     */
    static create(hashedKey, fee) {
        if (!Buffer.isBuffer(hashedKey) && !hashedKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
            throw Error('Invalid secret format.')

        const challenge = new Challenge(Buffer.alloc(Challenge.SIZE))
        challenge._hashedKey = hashedKey
        challenge._fee = fee

        return challenge
    }

    /**
     * Verifies the challenge by checking whether the given public matches the
     * one restored from the signature.
     * 
     * @param {PeerId} peerId PeerId instance that contains the public key of
     * the signer
     * @param {Buffer} secret the secret that was used to derive the key half
     */
    verify(peerId) {
        if (!peerId.pubKey)
            throw Error('Unable to verify challenge without a public key.')

        return this.counterparty.equals(peerId.pubKey.marshal())
    }
}

module.exports = Challenge