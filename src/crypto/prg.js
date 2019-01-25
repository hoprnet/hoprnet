'use strict'

const crypto = require('crypto')
const { numberToBuffer } = require('../utils')

const BLOCK_LENGTH = 16
const KEY_LENGTH = BLOCK_LENGTH
const IV_LENGTH = 12
const COUNTER_LENGTH = 4

const PRG_ALGORITHM = 'aes-128-ctr'

class PRG {
    constructor(key, iv) {
        this.key = key
        this.iv = iv

        this.initialised = true
    }

    static get IV_LENGTH() {
        return IV_LENGTH
    }

    static get KEY_LENGTH() {
        return KEY_LENGTH
    }

    static createPRG(key, iv) {
        if (!Buffer.isBuffer(key) || !Buffer.isBuffer(iv))
            throw Error(`Invalid input parameters. Got (${typeof key},${typeof iv}) instead of (Buffer, Buffer).`)

        if (key.length != KEY_LENGTH || iv.length != IV_LENGTH)
            throw Error(`Invalid input parameters. Expected a key of ${KEY_LENGTH} bytes and an initialization vector of ${IV_LENGTH} bytes.`)

        return new PRG(key, iv)
    }

    digest(start, end) {
        if (!this.initialised)
            throw Error('Module not initialized. Please do that first.')

        const firstBlock = Math.floor(start / BLOCK_LENGTH)
        const startOffset = start % BLOCK_LENGTH

        const lastBlock = Math.ceil(end / BLOCK_LENGTH)
        const lastBlockSize = end % BLOCK_LENGTH

        const amountOfBlocks = lastBlock - firstBlock

        const iv = Buffer.concat([this.iv, numberToBuffer(firstBlock, COUNTER_LENGTH)], IV_LENGTH + COUNTER_LENGTH)

        return crypto
            .createCipheriv(PRG_ALGORITHM, this.key, iv)
            .update(Buffer.alloc(amountOfBlocks * BLOCK_LENGTH, 0))
            .slice(startOffset, amountOfBlocks * BLOCK_LENGTH - (lastBlockSize > 0 ? BLOCK_LENGTH - lastBlockSize : 0))
    }
}

module.exports = PRG