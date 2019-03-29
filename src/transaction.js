'use strict'

const secp256k1 = require('secp256k1')

const { hash, numberToBuffer, bufferToNumber, getId, pubKeyToEthereumAddress } = require('./utils')

const SIGNATURE_LENGTH = 64
const VALUE_LENGTH = 32
const INDEX_LENGTH = 16
const NONCE_LENGTH = 16
const RECOVERY_LENGTH = 1
const CURVE_POINT_LENGTH = 33

// Format:
// 64 byte signature
// 16 byte nonce
// 16 byte index
// 32 byte value
// 1  byte signature recovery

class Transaction {
    constructor(buf = Buffer.alloc(Transaction.SIZE)) {
        this.buffer = buf
    }

    static get SIGNATURE_LENGTH() {
        return SIGNATURE_LENGTH
    }

    static get VALUE_LENGTH() {
        return VALUE_LENGTH
    }

    static get INDEX_LENGTH() {
        return INDEX_LENGTH
    }

    static get NONCE_LENGTH() {
        return NONCE_LENGTH
    }

    static get RECOVERY_LENGTH() {
        return RECOVERY_LENGTH
    }
    /**
     * @returns {Number} size of a transaction
     */
    static get SIZE() {
        return SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + RECOVERY_LENGTH + CURVE_POINT_LENGTH
    }

    // ========= Getters =========
    /**
     * @returns {Buffer} signature of the transaction
     */
    get signature() {
        return this.buffer.slice(0, SIGNATURE_LENGTH)
    }

    /**
     * @returns {Buffer} nonce of the transaction
     */
    get nonce() {
        return this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH)
    }

    /**
     * @returns {Buffer} index of the transaction
     */
    get index() {
        return this.buffer.slice(SIGNATURE_LENGTH + NONCE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH)
    }

    /**
     * @returns {Buffer} value resp. balance of the (update) transaction
     */
    get value() {
        return this.buffer.slice(SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH)
    }

    get curvePoint() {
        return this.buffer.slice(SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + CURVE_POINT_LENGTH)
    }
    /**
     * @returns {Buffer} recovery value that is necessary to recover the public key
     * that was used to create the signature
     */
    get recovery() {
        return this.buffer.slice(SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + CURVE_POINT_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + CURVE_POINT_LENGTH + RECOVERY_LENGTH)
    }

    // ========= Setters =========
    /**
     * @param {Buffer} newValue the balance of the transaction
     */
    set value(newValue) {
        delete this._counterparty
        delete this._hash

        this.value.fill(newValue, 0, VALUE_LENGTH)
    }

    /**
     * @param {Buffer} newSignature the signature of the transaction
     */
    set signature(newSignature) {
        delete this._counterparty
        delete this._hash

        this.signature.fill(newSignature, 0, SIGNATURE_LENGTH)
    }
    /**
     * @param {Number} newIndex the index of the transaction
     */
    set index(newIndex) {
        if (typeof newIndex === 'number')
            newIndex = numberToBuffer(newIndex)

        if (!Buffer.isBuffer(newIndex))
            throw Error(`Invalid input value. Expected a number or a buffer but got ${typeof newIndex}.`)

        delete this._counterparty
        delete this._hash

        this.index.fill(newIndex, 0, INDEX_LENGTH)
    }

    /**
     * @param {Buffer} nonce the nonce of the transaction
     */
    set nonce(nonce) {
        delete this._counterparty

        this.nonce.fill(nonce, 0, NONCE_LENGTH)
    }

    /**
     * @param {Buffer | Number} newRecovery
     */
    set recovery(newRecovery) {
        if (typeof newRecovery === 'number')
            newRecovery = numberToBuffer(newRecovery, RECOVERY_LENGTH)

        if (!Buffer.isBuffer(newRecovery))
            throw Error('Unable to parse input to Buffer.')

        delete this._counterparty
        delete this._hash

        this.recovery.fill(newRecovery, 0, RECOVERY_LENGTH)
    }

    set curvePoint(curvePoint) {
        this.curvePoint.fill(curvePoint, 0, CURVE_POINT_LENGTH)
    }

    // ==== Derived properties ===
    /**
     * @returns {Buffer} the hash of the transaction upon which the signature
     * will be computed
     */
    get hash() {
        if (this._hash)
            return this._hash

        this._hash = hash(this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + CURVE_POINT_LENGTH))

        return this._hash
    }

    /**
     * Returns the channelId. It tries to derive the public key from the embedded
     * signature and uses that key in combination with the given `peerId` to derive
     * the `channelId`
     * 
     * @param {PeerId} peerId peerId of the node
     * @returns {Buffer} the channelId
     */
    getChannelId(peerId) {
        return getId(
            pubKeyToEthereumAddress(peerId.pubKey.marshal()),
            pubKeyToEthereumAddress(this.counterparty)
        )
    }

    /**
     * @returns {Buffer} the public key as a compressed curve point
     */
    get counterparty() {
        if (this._counterparty)
            return this._counterparty

        this._counterparty = secp256k1.recover(this.hash, this.signature, bufferToNumber(this.recovery))

        return this._counterparty
    }

    // ========= Methods =========
    /**
     * Signs the transaction.
     * 
     * @param {PeerId} peerId a peerId that contains the private key to sign
     * the transaction
     */
    sign(peerId) {
        if (!peerId.privKey)
            throw Error('No private key found. Please provide one to sign the transaction.')

        const signature = secp256k1.sign(this.hash, peerId.privKey.marshal())

        this.signature = signature.signature
        this.recovery = numberToBuffer(signature.recovery, RECOVERY_LENGTH)

        return this
    }

    /**
     * Verifies the transaction.
     * 
     * @param {PeerId} peerId peerId that contains the public key of the
     * party that signed the transaction.
     * @returns {Boolean} whether the signature is valid 
     */
    // verify(peerId) {
    //     return this.counterparty.compare(peerId.pubKey.marshal()) === 0
    // }

    /**
     * Creates a Transaction instance from a Buffer.
     * 
     * @param {Buffer} buf the buffer representation
     * @returns {Transaction} Transaction instance
     */
    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length !== Transaction.SIZE)
            throw Error(`Invalid input argument. Expected a buffer of size ${Transaction.SIZE}.`)

        return new Transaction(buf)
    }

    /**
     * Exports Transaction as a Buffer.
     * @returns {Buffer} the transaction as Buffer
     */
    toBuffer() {
        return this.buffer
    }
}

module.exports = Transaction