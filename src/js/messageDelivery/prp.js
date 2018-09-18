'use strict'

const chacha = require('chacha')
const withIs = require('class-is')
// const blake2 = require('blake2')
const crypto = require('blake2')
const { bufferXOR_in_place } = require('../utils')

const KEY_LENGTH = 128 // Bytes
const IV_LENGTH = 48 // Bytes
const MIN_LENGTH = 32 // Bytes
const HASH_ALGORITHM = 'blake2sp'

module.exports.KEY_LENGTH = KEY_LENGTH
module.exports.IV_LENGTH = IV_LENGTH
module.exports.MIN_LENGTH = MIN_LENGTH

class PRP {
    constructor(key, iv) {
        if (!Buffer.isBuffer(key) || key.length != KEY_LENGTH)
            throw Error('Invalid key. Expected Buffer of size ' + KEY_LENGTH + ' bytes.')

        if (!Buffer.isBuffer(iv) || iv.length != IV_LENGTH)
            throw Error('Invalid initialisation vector. Expected Buffer of size ' + IV_LENGTH + ' bytes.')


        this.k1 = key.slice(0, 32)
        this.k2 = key.slice(32, 64)
        this.k3 = key.slice(64, 96)
        this.k4 = key.slice(96, 128)

        this.iv1 = iv.slice(0, 12)
        this.iv2 = iv.slice(12, 24)
        this.iv3 = iv.slice(24, 36)
        this.iv4 = iv.slice(36, 48)

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

        let l = plaintext.slice(0, 32)
        let r = plaintext.slice(32)

        let final

        const cipher1 = chacha.chacha20(bufferXOR_in_place(this.k1, l), this.iv1);
        r = cipher1.update(r)
        final = cipher1.final()
        r = Buffer.concat([r, final], r.length + final.length)

        const hash1 = crypto.createHash(HASH_ALGORITHM)
        hash1.update(Buffer.concat([this.k2, this.iv2, r]))
        l = bufferXOR_in_place(l, hash1.digest())

        const cipher2 = chacha.chacha20(bufferXOR_in_place(this.k3, l), this.iv3);
        r = cipher2.update(r)
        final = cipher2.final()
        r = Buffer.concat([r, final], r.length + final.length)

        const hash2 = crypto.createHash(HASH_ALGORITHM)
        hash2.update(Buffer.concat([this.k4, this.iv4, r]))
        l = bufferXOR_in_place(l, hash2.digest())

        return Buffer.concat([l, r], l.length + r.length)
    }

    inverse(ciphertext) {
        if (!this.initialised)
            throw Error('Uninitialised. Provide key and iv first.')

        if (!Buffer.isBuffer(ciphertext))
            throw Error('Expected buffers. Got ' + typeof plaintext)

        if (ciphertext.length < MIN_LENGTH)
            throw Error('Expected ciphertext with a length of a least ' + MIN_LENGTH + ' bytes. Got ' + ciphertext.length)

        let l = ciphertext.slice(0, 32)
        let r = ciphertext.slice(32)

        let final

        const hash2 = crypto.createHash(HASH_ALGORITHM)
        hash2.update(Buffer.concat([this.k4, this.iv4, r]))
        l = bufferXOR_in_place(l, hash2.digest())

        const cipher2 = chacha.chacha20(bufferXOR_in_place(this.k3, l), this.iv3);
        r = cipher2.update(r)
        final = cipher2.final()
        r = Buffer.concat([r, final], r.length + final.length)

        const hash1 = crypto.createHash(HASH_ALGORITHM)
        hash1.update(Buffer.concat([this.k2, this.iv2, r]))
        l = bufferXOR_in_place(l, hash1.digest())

        const cipher1 = chacha.chacha20(bufferXOR_in_place(this.k1, l), this.iv1);
        r = cipher1.update(r)
        final = cipher1.final()
        r = Buffer.concat([r, final], r.length + final.length)

        return Buffer.concat([l, r], l.length + r.length)
    }
}
module.exports = withIs(PRP, { className: 'PRP' })