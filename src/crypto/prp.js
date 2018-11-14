'use strict'

const chacha = require('chacha')
const withIs = require('class-is')
const crypto = require('blake2')
const { bufferXOR } = require('../utils')

const INTERMEDIATE_KEY_LENGTH = 32
const INTERMEDIATE_IV_LENGTH = 12

const HASH_LENGTH = 32
const KEY_LENGTH = 4 * INTERMEDIATE_KEY_LENGTH // 128 Bytes
const IV_LENGTH = 4 * INTERMEDIATE_IV_LENGTH // Bytes
const MIN_LENGTH = HASH_LENGTH // Bytes
const HASH_ALGORITHM = 'blake2bp'

class PRP {
    constructor(key, iv) {
        if (!Buffer.isBuffer(key) || key.length != KEY_LENGTH)
            throw Error('Invalid key. Expected Buffer of size ' + KEY_LENGTH + ' bytes.')

        if (!Buffer.isBuffer(iv) || iv.length != IV_LENGTH)
            throw Error('Invalid initialisation vector. Expected Buffer of size ' + IV_LENGTH + ' bytes.')

        this.k1 = key.slice(0, INTERMEDIATE_KEY_LENGTH)
        this.k2 = key.slice(INTERMEDIATE_KEY_LENGTH, 2 * INTERMEDIATE_KEY_LENGTH)
        this.k3 = key.slice(2 * INTERMEDIATE_KEY_LENGTH, 3 * INTERMEDIATE_KEY_LENGTH)
        this.k4 = key.slice(3 * INTERMEDIATE_KEY_LENGTH, 4 * INTERMEDIATE_KEY_LENGTH)

        this.iv1 = iv.slice(0, INTERMEDIATE_IV_LENGTH)
        this.iv2 = iv.slice(INTERMEDIATE_IV_LENGTH, 2 * INTERMEDIATE_IV_LENGTH)
        this.iv3 = iv.slice(2 * INTERMEDIATE_IV_LENGTH, 3 * INTERMEDIATE_IV_LENGTH)
        this.iv4 = iv.slice(3 * INTERMEDIATE_IV_LENGTH, 4 * INTERMEDIATE_IV_LENGTH)

        this.initialised = true
    }

    static get KEY_LENGTH() {
        return KEY_LENGTH
    }

    static get IV_LENGTH() {
        return IV_LENGTH
    }

    static get MIN_LENGTH() {
        return MIN_LENGTH
    }

    static createPRP(key, iv) {
        return new PRP(key, iv)
    }

    permutate(plaintext) {
        if (!this.initialised)
            throw Error('Uninitialised. Provide key and iv first.')

        if (!Buffer.isBuffer(plaintext))
            throw Error('Expected buffers. Got ' + typeof plaintext)

        if (plaintext.length < MIN_LENGTH)
            throw Error('Expected plaintext with a length of a least ' + MIN_LENGTH + ' bytes. Got ' + plaintext.length)

        const data = plaintext

        encrypt(data, this.k1, this.iv1)
        hash(data, this.k2, this.iv2)
        encrypt(data, this.k3, this.iv3)
        hash(data, this.k4, this.iv4)

        return data
    }

    inverse(ciphertext) {
        if (!this.initialised)
            throw Error('Uninitialised. Provide key and iv first.')

        if (!Buffer.isBuffer(ciphertext))
            throw Error('Expected buffers. Got ' + typeof plaintext)

        if (ciphertext.length < MIN_LENGTH)
            throw Error('Expected ciphertext with a length of a least ' + MIN_LENGTH + ' bytes. Got ' + ciphertext.length)

        const data = ciphertext

        hash(data, this.k4, this.iv4)
        encrypt(data, this.k3, this.iv3)
        hash(data, this.k2, this.iv2)
        encrypt(data, this.k1, this.iv1)

        return data
    }
}

function hash(data, k, iv) {
    const hash = crypto.createKeyedHash(
        HASH_ALGORITHM,
        Buffer.concat([k, iv], INTERMEDIATE_KEY_LENGTH + INTERMEDIATE_IV_LENGTH),
        {digestLength: 32}
    )
    hash.update(data.slice(HASH_LENGTH))

    data
        .fill(bufferXOR(data.slice(0, HASH_LENGTH), hash.digest()), 0, HASH_LENGTH)
}

function encrypt(data, k, iv) {
    const cipher = chacha.chacha20(bufferXOR(k, data.slice(0, HASH_LENGTH)), iv);

    const ciphertext = cipher.update(data.slice(HASH_LENGTH))
    ciphertext.copy(data, HASH_LENGTH)
}

module.exports = withIs(PRP, { className: 'PRP' })
