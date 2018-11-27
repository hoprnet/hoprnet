'use strict'

const { sha3 } = require('web3').utils
const { randomBytes } = require('crypto')

// ==========================
// General methods
// ==========================

module.exports.hash = function (buf) {
    if (!Buffer.isBuffer(buf))
        throw Error('Invalid input. Please use a Buffer')

    return Buffer.from(sha3(buf).slice(2), 'hex')
}

module.exports.deepCopy = function (instance, Class) {
    if (typeof instance.toBuffer !== 'function' || !['function', 'number'].includes(typeof Class.SIZE) || typeof Class.fromBuffer !== 'function')
        throw Error('Invalid object.')

    const buf = Buffer.alloc(Class.SIZE)
        .fill(instance.toBuffer(), 0, Class.SIZE)

    return Class.fromBuffer(buf)
}

module.exports.parseJSON = function (str) {
    return JSON.parse(str, (key, value) => {
        if (value && value.type === 'Buffer') {
            return Buffer.from(value.data)
        }

        return value
    })
}

// ==========================
// Buffer methods
// ==========================

module.exports.bufferADD = function (buf1, buf2) {
    if (!Buffer.isBuffer(buf1))
        throw Error('Expected a buffer. Got \"' + typeof buf1 + '\" instead.')

    const a = Number.parseInt(buf1.toString('hex'))
    let b, length

    if (Buffer.isBuffer(buf2)) {
        // Incorrect hex format ?
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

// ==========================
// Collection methods
// ==========================

module.exports.randomSubset = function (array, subsetSize, filter = _ => true) {
    if (!Number.isInteger(subsetSize) || subsetSize < 0)
        throw Error('Invalid input arguments. Please provide a positive subset size. Got \"' + subsetSize + '\" instead.')

    if (!array || !Array.isArray(array))
        throw Error('Invalid input parameters. Expected an Array. Got \"' + typeof array + '\" instead.')

    if (subsetSize > array.length)
        throw Error('Invalid subset size. Subset size must not be greater than set size.')

    if (subsetSize == 0)
        return []

    if (subsetSize === array.length)
        return module.exports.randomPermutation(array)

    const byteAmount = Math.max(Math.ceil(Math.log2(array.length)) / 8, 1)

    const indexSet = new Set()

    while (indexSet.size < subsetSize) {
        const index = module.exports.bufferToNumber(randomBytes(byteAmount)) % array.length

        if (filter(array[index]))
            indexSet.add(index)
    }

    const res = []
    indexSet.forEach(index => {
        res.push(array[index])
    })

    return res
}

module.exports.randomPermutation = function (array) {
    // TODO
    return array
}

// ==========================
// PeerId methods
// ==========================

const PeerId = require('peer-id')
const PREFIX = 0x12
const c = require('./constants')
const p = require('./packet/header/parameters')
const Multihash = require('multihashes')

module.exports.keyPairToPeerId = function (key) {
    return new PeerId(Multihash.encode(key.public.marshal(), PREFIX), key, key.public)
}

module.exports.pubKeyToPeerId = function (buf) {
    if (!Buffer.isBuffer(buf) || buf.length !== p.COMPRESSED_PUBLIC_KEY_LENGTH)
        throw Error('Invalid input parameter. Expected a Buffer of size ' + p.COMPRESSED_PUBLIC_KEY_LENGTH + '. Got ' + typeof buf + ' instead.')
    
    return PeerId.createFromBytes(Multihash.encode(key.public.marshal(), PREFIX))
}

module.exports.peerIdToPubKey = function (peerId) {
    return Multihash.decode(peerId.toBytes()).digest

    if (buf.length !== c.PEERID_LENGTH)
        throw Error('Invalid input argument.')

    if (buf.slice(0,1).compare(Buffer.from(PREFIX.slice(2), 'hex')) !== 0)
        throw Error('Invalid prefix')

    return buf.slice(1)
}