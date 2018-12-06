'use strict'

const forEachRight = require('lodash.foreachright')
const withIs = require('class-is')

const constants = require('../constants')
const Header = require('./header')
const PRP = require('../crypto/prp')

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

    static createMessage(msg, buffer = Buffer.alloc(Message.SIZE)) {
        if (!Buffer.isBuffer(msg))
            throw Error('Wrong input values. Expected a Buffer. Got \"' + typeof msg + '\" instead.')

        const msgLength = msg.length
        
        buffer
            .fill(msg, 0, msgLength)
            .fill(PADDING, msgLength, msgLength + PADDING_LENGTH)
            .fill(0, msgLength + PADDING_LENGTH, Message.SIZE)

        return new Message(buffer)
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

    decrypt(secret, cb) {
        const { key, iv } = Header.deriveCipherParameters(secret)

        PRP.createPRP(key, iv).inverse(this.buffer)

        cb(null, this)
    }
}

module.exports = withIs(Message, { className: 'Message', symbolName: '@validitylabs/hopper/Message' })