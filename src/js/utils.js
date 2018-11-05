'use strict'

const { sha3 } = require('web3').utils


module.exports.parseJSON = function (str) {
    return JSON.parse(str, (key, value) => {
        if (value && value.type === 'Buffer') {
            return Buffer.from(value.data)
        }

        return value
    })
}

module.exports.bufferXOR = function (buf1, buf2) {
    if (!Buffer.isBuffer(buf1) || !Buffer.isBuffer(buf2))
        throw Error('Input values have to be provided as Buffers. Got ' + typeof buf1 + ' and ' + typeof buf2)

    if (buf1.length !== buf2.length)
        throw Error('Buffer must have the same length. Got buffers of length ' + buf1.length + ' and ' + buf2.length)

    return buf1.map((elem, index) => (elem ^ buf2[index]))
}

module.exports.numberToBuffer = function (i, length) {
    if (i < 0)
        throw Error('Not implemented!')

    return Buffer.from(i.toString(16).padStart(length * 2, '0'), 'hex')
}

module.exports.bufferToNumber = function (buf) {
    if (!Buffer.isBuffer(buf) || buf.length === 0)
        throw Error('Invalid input value. Expected a non-empty buffer.')

    return parseInt(buf.toString('hex'), 16)
}

module.exports.bufferADD = function (buf1, buf2) {
    if (!Buffer.isBuffer(buf1))
        throw Error('Expected a buffer. Got \"' + typeof buf1 + '\" instead.')

    const a = Number.parseInt(buf1.toString('hex'))
    let b, length

    if (Buffer.isBuffer(buf2)) {
        b = Number.parseInt(buf2.toString('hex'))
        length = Math.max(buf1.length, buf2.length)

    } else if (Number.isInteger(buf2)) {
        b = buf2
        length = buf1.length
    } else {
        throw Error('Invalid input values. Got \"' + typeof buf1 + '\" and \"' + typeof buf2 + '\".')
    }

    return module.exports.numberToBuffer(a + b, length)
}

module.exports.hash = function (buf) {
    if (!Buffer.isBuffer(buf))
        throw Error('Invalid input. Please use a Buffer')

    return Buffer.from(sha3(buf).slice(2), 'hex')
}