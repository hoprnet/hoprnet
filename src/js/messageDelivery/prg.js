'use strict'

const crypto = require('crypto')
const withIs = require('class-is')

const BLOCK_LENGTH = 16
const KEY_LENGTH = BLOCK_LENGTH
const IV_LENGTH = 12

const PRG_ALGORITHM = 'aes-128-ctr'

class PRG {
    constructor(key, iv, byteOffset) {
        this.key = key
        this.iv = iv
        this.byteOffset = byteOffset

        this.initialised = true
    }

    digest(size) {
        if (!this.initialised)
            throw Error('Uninitialised module.')

        if (size <= 0)
            throw Error('Expected a size strictly greater than 0. Got ' + size)

        return crypto
            .createCipheriv(PRG_ALGORITHM, this.key, this.iv)
            .update(
                Buffer.alloc(size).fill(0)
            )
            .slice(this.byteOffset || 0, size)
    }

    static get IV_LENGTH() {
        return IV_LENGTH
    }

    static get KEY_LENGTH() {
        return KEY_LENGTH
    }

    static get BLOCK_LENGTH() {
        return BLOCK_LENGTH
    }

    static createPRG(key, iv) {
        if (!Buffer.isBuffer(key) || key.length != KEY_LENGTH)
            throw Error('Invalid key. Expected a Buffer of size ' + KEY_LENGTH + '.')

        if (!iv.hasOwnProperty('iv'))
            throw Error('Missing initialisation vector.')

        if (!Buffer.isBuffer(iv.iv) || (iv.iv.length != IV_LENGTH && iv.iv.length != BLOCK_LENGTH))
            throw Error('Invalid initialisation vector. Expected a Buffer of size ' + IV_LENGTH + '.')

        if (iv.iv.length === IV_LENGTH)
            iv.iv = Buffer.concat([iv, Buffer.alloc(BLOCK_LENGTH - IV_LENGTH).fill(0)], BLOCK_LENGTH)

        return new PRG(key, iv.iv, iv.byteOffset)
    }
}

module.exports = withIs(PRG, { className: 'PRG' })