'use strict'

const { sha3, hexToBytes, toChecksumAddress } = require('web3').utils
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

module.exports.bufferADD = (buf1, buf2) => {
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

module.exports.bufferXOR = (buf1, buf2) => {
    if (!Buffer.isBuffer(buf1) || !Buffer.isBuffer(buf2))
        throw Error('Input values have to be provided as Buffers. Got ' + typeof buf1 + ' and ' + typeof buf2)

    if (buf1.length !== buf2.length)
        throw Error('Buffer must have the same length. Got buffers of length ' + buf1.length + ' and ' + buf2.length)

    return buf1.map((elem, index) => (elem ^ buf2[index]))
}

module.exports.numberToBuffer = (i, length) => {
    if (i < 0)
        throw Error('Not implemented!')

    return Buffer.from(i.toString(16).padStart(length * 2, '0'), 'hex')
}

module.exports.bufferToNumber = (buf) => {
    if (!Buffer.isBuffer(buf) || buf.length === 0)
        throw Error('Invalid input value. Expected a non-empty buffer.')

    return parseInt(buf.toString('hex'), 16)
}

// ==========================
// Collection methods
// ==========================

module.exports.randomSubset = (array, subsetSize, filter = _ => true) => {
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

    // if (subsetSize == 1) {
        
    //     return [array[module.exports.bufferToNumber(randomBytes(byteAmount)) % array.length]]
    // }

    let notChosen = new Set()
    let chosen = new Set()
    let found, breakUp = false
    
    let index = 0
    for (let i = 0; i < subsetSize && !breakUp; i++) {
        index = (index + module.exports.bufferToNumber(randomBytes(byteAmount))) % array.length
        
        found = false

        do {
            while(chosen.has(index) || notChosen.has(index)) {
                index = (index + 1) % array.length
            }

            if (!filter(array[index])) {
                notChosen.add(index)
                index = (index + 1) % array.length
                found = false
            } else {
                chosen.add(index)
                found = true
            }

            if (notChosen.size + chosen.size == array.length && chosen.size < subsetSize) {
                breakUp = true
                break
            }


        } while(!found)
    }

    const result = []
    for (let index of chosen) {
        result.push(array[index])
    }

    return result
}

// Implementation of the (optimized) Fisher-Yates
// shuffling algorithm
module.exports.randomPermutation = (array) => {
    if (array.length <= 1)
        return array
        
    let i, j, tmp

    const byteAmount = Math.max(Math.ceil(Math.log2(array.length)) / 8, 1)

    for (i = array.length - 1; i > 0; i--) {
        j = module.exports.bufferToNumber(randomBytes(byteAmount)) % (i + 1)
        tmp = array[i]
        array[i] = array[j]
        array[j] = tmp
    }

    return array
}

// TODO: Proper random number generation
// module.exports.randomNumber(start, end)

// ==========================
// Ethereum methods
// ==========================
const secp256k1 = require('secp256k1')

module.exports.pubKeyToEthereumAddress = (pubKey) => {
    const hash = sha3(secp256k1.publicKeyConvert(pubKey, false).slice(1))

    return toChecksumAddress(hash.slice(0,2).concat(hash.slice(26)))
}

module.exports.isPartyA = (sender, otherParty) => {
    if (typeof sender === 'string' && typeof otherParty === 'string') {
        return Buffer.compare(Buffer.from(hexToBytes(sender), 0, ETHEUREUM_ADDRESS_SIZE), Buffer.from(hexToBytes(otherParty), 0, ETHEUREUM_ADDRESS_SIZE)) < 0
    } else if (Buffer.isBuffer(sender) && Buffer.isBuffer(otherParty)) {
        return Buffer.compare(sender, otherParty) < 0
    } else {
        throw Error('Invalid argument')
    }
}

const ETHEUREUM_ADDRESS_SIZE = 20 // Bytes
module.exports.getId = (sender, otherParty) => {
    sender = Buffer.from(hexToBytes(sender), 0, ETHEUREUM_ADDRESS_SIZE)
    otherParty = Buffer.from(hexToBytes(otherParty), 0, ETHEUREUM_ADDRESS_SIZE)

    if (module.exports.isPartyA(sender, otherParty)) {
        // console.log('[\'' + sender.toString('base64') + '\ \'' + otherParty.toString('base64') + '\']: ChannelId: \'' + module.exports.hash(Buffer.concat([sender, otherParty], 2 * ETHEUREUM_ADDRESS_SIZE)).toString('base64') + '\' as party A.')
        return module.exports.hash(Buffer.concat([sender, otherParty], 2 * ETHEUREUM_ADDRESS_SIZE))
    } else {
        // console.log('[\'' + sender.toString('base64') + '\ \'' + otherParty.toString('base64') + '\']: ChannelId: \'' + module.exports.hash(Buffer.concat([otherParty, sender], 2 * ETHEUREUM_ADDRESS_SIZE)).toString('base64') + '\' as party B.')
        return module.exports.hash(Buffer.concat([otherParty, sender], 2 * ETHEUREUM_ADDRESS_SIZE))
    }
}