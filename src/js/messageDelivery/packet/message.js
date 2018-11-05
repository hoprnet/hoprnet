'use strict'

const forEachRight = require('lodash.foreachright')
const withIs = require('class-is')

const constants = require('../constants')
const Header = require('./header')
const PRP = require('../prp')

const PADDING = Buffer.from('PADDING')
const PADDING_LENGTH = PADDING.length

class Message {
    constructor(buf) {
        this.buffer = buf
    }

    toBuffer() {
        return this.buffer
    }

    static get SIZE() {
        return constants.PACKET_SIZE + PADDING_LENGTH
    }

    get plaintext() {
        const lastIndex = this.buffer.lastIndexOf(PADDING)

        if (lastIndex < 0)
            throw Error('String does not contain a valid padding.')

        return this.buffer.slice(0, lastIndex)
    }

    static createMessage(msg) {
        return new Message(
            Buffer.concat(
                [
                    msg,
                    PADDING,
                    Buffer.alloc(constants.PACKET_SIZE - msg.length).fill(0)
                ],
                Message.SIZE)
        )
    }

    static fromBuffer(buf) {
        if (!Buffer.isBuffer(buf) || buf.length !== Message.SIZE)
            throw Error('Invalid input values. Expected a buffer of ' + Message.SIZE + ' bytes.')

        return new Message(buf)
    }

    onionEncrypt(secrets) {
        if (!Array.isArray(secrets) || secrets.length <= 0)
            throw Error('Invald input arguments. Expected array with at least one secret key.')

        forEachRight(secrets, (secret) => {
            const { key, iv } = Header.deriveCipherParameters(secret)

            PRP.createPRP(key, iv).permutate(this.buffer)
        })

        return this
    }

    decrypt(secret) {
        const { key, iv } = Header.deriveCipherParameters(secret)

        PRP.createPRP(key, iv).inverse(this.buffer)
    }
}

module.exports = withIs(Message, { className: 'Message', symbolName: '@validitylabs/hopper/Message' })