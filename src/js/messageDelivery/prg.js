'use strict'

const crypto = require('crypto')
const withIs = require('class-is')

const BLOCK_LENGTH = 16
const PRG_KEY_LENGTH = BLOCK_LENGTH
const PRG_IV_LENGTH = 12

const PRG_ALGORITHM = 'aes-128-ctr'

class PRG {
    constructor(key, iv) {
        this.key = key
        this.iv = iv

        this.initialised = true
    }

    digest(size) {
        if (!this.initialised)
            throw Error('Uninitialised module.')
        if (size <= 0)
            throw Error('Expected a non-zero size. Got ' + size)

        return crypto
            .createCipheriv(PRG_ALGORITHM, this.key, this.iv)
            .update(
                Buffer.alloc(size).fill(0)
            )
    }

    static get PRG_IV_LENGTH() {
        return PRG_IV_LENGTH
    }

    static get PRG_KEY_LENGTH() {
        return PRG_KEY_LENGTH
    }

    static createPRG(key, iv) {
        if (!Buffer.isBuffer(key) || key.length != PRG_KEY_LENGTH)
            throw Error('Invalid key. Expected a Buffer of size ' + PRG_KEY_LENGTH + '.')

        if (!Buffer.isBuffer(iv) || (iv.length != PRG_IV_LENGTH && iv.length != BLOCK_LENGTH))
            throw Error('Invalid initialisation vector. Expected a Buffer of size ' + PRG_IV_LENGTH + '.')

        if (iv.length === PRG_IV_LENGTH)
            iv = Buffer.concat([iv, Buffer.alloc(BLOCK_LENGTH - PRG_IV_LENGTH).fill(0)], BLOCK_LENGTH)

        return new PRG(key, iv)
    }
}
module.exports = withIs(PRG, { className: 'PRG' })