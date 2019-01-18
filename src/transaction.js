'use strict'

const { recover, sign } = require('secp256k1')

const { hash, numberToBuffer, bufferToNumber, bufferXOR, getId, pubKeyToEthereumAddress } = require('./utils')

const SIGNATURE_LENGTH = 64
const KEY_LENGTH = 32
const VALUE_LENGTH = 32
const INDEX_LENGTH = 16
const NONCE_LENGTH = 16
const RECOVERY_LENGTH = 1

// Format:
// 64 byte signature
// 16 byte nonce
// 16 byte index
// 32 byte value
// 1  byte signature recovery

class Transaction {
    constructor(buf = Buffer.alloc(Transaction.SIZE), encrypted = false) {
        this.buffer = buf
        this.encrypted = encrypted
    }

    static get SIGNATURE_LENGTH() {
        return SIGNATURE_LENGTH
    }

    static get KEY_LENGTH() {
        return KEY_LENGTH
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
        return SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + RECOVERY_LENGTH
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

    /**
     * @returns {Buffer} recovery value that is necessary to recover the public key
     * that was used to create the signature
     */
    get recovery() {
        return this.buffer.slice(SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH + RECOVERY_LENGTH)
    }

    // ========= Setters =========
    /**
     * @param {Buffer} newValue the balance of the transaction
     */
    set value(newValue) {
        this.value.fill(newValue, 0, VALUE_LENGTH)
    }

    /**
     * @param {Buffer} newSignature the signature of the transaction
     */
    set signature(newSignature) {
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

        this.index.fill(newIndex, 0, INDEX_LENGTH)
    }
    // set index(newIndex) {
    //     this.buffer
    //         .slice(SIGNATURE_LENGTH + NONCE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH)
    //         .fill(numberToBuffer(newIndex, INDEX_LENGTH), 0, INDEX_LENGTH)
    // }

    /**
     * @param {Buffer} nonce the nonce of the transaction
     */
    set nonce(nonce) {
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

        this.recovery.fill(newRecovery, 0, RECOVERY_LENGTH)
    }

    // ==== Derived properties ===
    /**
     * @returns {Buffer} the hash of the transaction upon which the signature
     * will be computed
     */
    get hash() {
        return hash(this.buffer.slice(SIGNATURE_LENGTH, SIGNATURE_LENGTH + NONCE_LENGTH + INDEX_LENGTH + VALUE_LENGTH))
    }

    /**
     * Returns by using the peerId of that node the channelId of that payment
     * channel.
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
        return recover(this.hash, this.signature, bufferToNumber(this.recovery))
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

        const signature = sign(this.hash, peerId.privKey.marshal())

        this.signature = signature.signature
        this.recovery = numberToBuffer(signature.recovery, RECOVERY_LENGTH)
    }

    /**
     * Verifies the transaction.
     * 
     * @param {PeerId} peerId peerId that contains the public key of the
     * party that signed the transaction.
     * @returns {Boolean} whether the signature is valid 
     */
    verify(peerId) {
        return this.counterparty.compare(peerId.pubKey.marshal()) === 0
    }

    /**
     * Creates a Transaction instance from a Buffer.
     * 
     * @param {Buffer} buf the buffer representation
     * @param {Boolean} encrypted 'true' if transaction is encrypted, default 'false'
     * @returns {Transaction} Transaction instance
     */
    static fromBuffer(buf, encrypted = false) {
        if (!Buffer.isBuffer(buf) || buf.length !== Transaction.SIZE)
            throw Error(`Invalid input argument. Expected a buffer of size ${Transaction.SIZE}.`)

        return new Transaction(buf, encrypted)
    }

    /**
     * Exports Transaction as a Buffer.
     * @returns {Buffer} the transaction as Buffer
     */
    toBuffer() {
        return this.buffer
    }

    /**
     * Encrypts the signature of the transaction.
     * 
     * @param {Buffer} key the key that is used to encrypt the transaction
     */
    encrypt(key) {
        if (!Buffer.isBuffer(key) || key.length !== KEY_LENGTH)
            throw Error('Invalid key.')

        if (this.encrypted)
            throw Error('Cannot encrypt an already encrypted transaction.')

        this.encrypted = true
        this.signature.fill(bufferXOR(Buffer.concat([key, key], 2 * KEY_LENGTH), this.signature), 0, SIGNATURE_LENGTH)

        return this
    }

    /**
     * Decrypts the signature of the transaction
     * 
     * @param {Buffer} key the key that is used to decrypt the transaction
     */
    decrypt(key) {
        if (!Buffer.isBuffer(key) || key.length !== KEY_LENGTH)
            throw Error('Invalid key.')

        if (!this.encrypted)
            throw Error('Cannot decrypt a transaction more than once.')

        this.encrypted = false
        this.signature.fill(bufferXOR(Buffer.concat([key, key], 2 * KEY_LENGTH), this.signature), 0, SIGNATURE_LENGTH)

        return this
    }
}

module.exports = Transaction