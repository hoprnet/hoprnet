'use strict'

const { sha3, hexToBytes, toChecksumAddress, toWei } = require('web3').utils
const { randomBytes } = require('crypto')

// ==========================
// General methods
// ==========================

module.exports.hash = (buf) => {
    if (!Buffer.isBuffer(buf))
        throw Error('Invalid input. Please use a Buffer')

    return Buffer.from(sha3(buf).slice(2), 'hex')
}
/**
 * Generate deep Copy of an instance
 * @param  {} instance instance of T
 * @param  {} Class T
 */
module.exports.deepCopy = (instance, Class) => {
    if (typeof instance.toBuffer !== 'function' || !['function', 'number'].includes(typeof Class.SIZE) || typeof Class.fromBuffer !== 'function')
        throw Error('Invalid object.')

    const buf = Buffer.alloc(Class.SIZE)
        .fill(instance.toBuffer(), 0, Class.SIZE)

    return Class.fromBuffer(buf)
}

/**
 * Parse JSON while recovering all Buffer elements
 * @param  {String} str JSON string
 */
module.exports.parseJSON = (str) =>
    JSON.parse(str || '', (key, value) => {
        if (value && value.type === 'Buffer') {
            return Buffer.from(value.data)
        }

        return value
    })

// ==========================
// Buffer methods
// ==========================
/**
 * result = buf1 + buf2
 * @param  {Buffer} buf1
 * @param  {Buffer} buf2
 */
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
/**
 * Bitwise XOR of two Buffers.
 * 
 * @param  {Buffer} buf1 first Buffer
 * @param  {Buffer} buf2 second Buffer
 * 
 * @returns {Buffer} @param buf1 ^ @param buf2
 */
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
/**
 * Picks @param subsetSize elements at random from @param array .
 * The order of the picked elements does not coincide with their
 * order in @param array
 * 
 * @param  {Array} array the array to pick the elements from
 * @param  {Number} subsetSize the requested size of the subset
 * @param  {Function} filter
 * 
 * @returns {Array} array with at most @param subsetSize elements
 * that pass the test.
 * 
 * @notice If less than @param subsetSize elements pass the test,
 * the result will contain less than @param subsetSize elements. 
 */
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
        // Returns a random permutation of all elements that pass
        // the test
        return module.exports.randomPermutation(array.filter(filter))

    const byteAmount = Math.max(Math.ceil(Math.log2(array.length)) / 8, 1)

    if (subsetSize == 1) {
        let i = 0
        let index = module.exports.bufferToNumber(randomBytes(byteAmount)) % array.length
        while (!filter(array[index])) {
            if (i === array.length) {
                // There seems to be no element in the array
                // that passes the test.
                return []
            }
            i++
            index = (index + 1) % array.length
        }
        return [array[index]]
    }

    let notChosen = new Set()
    let chosen = new Set()
    let found, breakUp = false

    let index = 0
    for (let i = 0; i < subsetSize && !breakUp; i++) {
        index = (index + module.exports.bufferToNumber(randomBytes(byteAmount))) % array.length

        found = false

        do {
            while (chosen.has(index) || notChosen.has(index)) {
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


        } while (!found)
    }

    const result = []
    for (let index of chosen) {
        result.push(array[index])
    }

    return result
}

/**
 * Return a random permutation of the given @param array
 * by using the (optimized) Fisher-Yates shuffling algorithm.
 * 
 * @param  {Array} array the array to permutate
 */
module.exports.randomPermutation = (array) => {
    if (!Array.isArray(array))
        throw Error('Invalid input parameters. Got \'' + typeof array + '\' instead of Buffer.')

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
const { publicKeyConvert } = require('secp256k1')
/**
 * Derives an Ethereum address from the given public key.
 * 
 * @param  {Buffer} pubKey given as compressed elliptic curve point.
 * 
 * @returns {String} e.g. 0xc1912fEE45d61C87Cc5EA59DaE31190FFFFf232d
 */
module.exports.pubKeyToEthereumAddress = (pubKey) => {
    const hash = sha3(publicKeyConvert(pubKey, false).slice(1))

    return toChecksumAddress(hash.slice(0, 2).concat(hash.slice(26)))
}

/**
 * Checks whether the ethereum address of the @param sender is
 * smaller than the ethereum address of the @param otherParty
 * 
 * @param {String | Buffer} sender an ethereum address
 * @param {String | Buffer} otherParty another ethereum address
 */
module.exports.isPartyA = (sender, otherParty) => {
    if (typeof sender === 'string') {
        if (sender.length !== 42)
            throw Error('Invalid input parameters')

        sender = Buffer.from(sender.slice(2), 'hex')
    }

    if (typeof otherParty === 'string') {
        if (otherParty.length !== 42) {
            throw Error('Invalid input parameters')
        }
        otherParty = Buffer.from(otherParty.slice(2), 'hex')
    }

    if (!Buffer.isBuffer(sender, otherParty))
        throw Error('Invalid input parameters')

    if (sender.length !== 20 || otherParty.length !== 20)
        throw Error('Invalid input parameters')

    return Buffer.compare(sender, otherParty) < 0
}

const ETHEUREUM_ADDRESS_SIZE = 20 // Bytes

/**
 * Computes the ID that is used by the smart contract to 
 * identify the payment channels.
 * 
 * @param {String} sender an ethereum address
 * @param {String} otherParty another ethereum address
 */
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

// ==========================
// libp2p methods
// ==========================
const libp2p_crypto = require('libp2p-crypto').keys
const PeerId = require('peer-id')

module.exports.pubKeyToPeerId = (pubKey, cb) =>
    PeerId.createFromPubKey(new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PublicKey(pubKey).bytes, cb)


// module.exports.privKeyToPeerId = (privKey, cb) =>
//     PeerId.createFromPrivKey(new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PrivateKey(privKey).bytes, cb)

// ==========================
// Ganache-core methods   <-- ONLY FOR TESTING
// ==========================
const { waterfall, during } = require('async')

const resetColor = "\x1b[0m"
const blueText = "\x1b[34m"
/**
 * Mine a single block
 * 
 * @param {Object} provider a valid Web3 provider
 */
module.exports.mineBlock = (provider) => waterfall([
    (cb) => provider.send({
        jsonrpc: '2.0',
        method: 'evm_increaseTime',
        params: [123],
        id: Date.now(),
    }, (err, result) => cb(err)),
    (cb) => provider.send({
        jsonrpc: '2.0',
        method: 'evm_mine',
        id: Date.now(),
    }, (err, result) => cb(err)),
    () => provider.send({
        jsonrpc: '2.0',
        method: 'eth_blockNumber',
        id: Date.now()
    }, (err, response) => {
        if (err) { throw err }

        console.log('%sNow on block %d.%s', blueText, parseInt(response.result, 16), resetColor)
    })
])

// /**
//  * Go to a specific block by mining probably empty blocks
//  * 
//  * @param {Object} provider a valid Web3 provider
//  * @param {Number} blockNumber the block to go to
//  */
// module.exports.gotoBlock = (provider, blockNumber, cb) => {

//     console.log('%sGoing to block %d by mining probably empty blocks%s', blueText, blockNumber, resetColor)

//     during(
//         (cb) => provider.send({
//             jsonrpc: '2.0',
//             method: 'eth_blockNumber',
//             id: Date.now()
//         }, (err, response) => {
//             console.log('Block ' + parseInt(response.result, 16))
//             cb(err, parseInt(response.result, 16) < blockNumber)
//         }),
//         (cb) => provider.send({
//             jsonrpc: '2.0',
//             method: 'evm_mine',
//             id: Date.now(),
//         }, cb),
//         cb)
// }

// ==========================
// Web3.js methods
// ==========================

module.exports.sendTransaction = (tx, peerId, web3, cb) =>
    web3.eth.accounts.privateKeyToAccount(
        typeof peerId === 'string' ?
            peerId :
            '0x'.concat(peerId.privKey.marshal().toString('hex'))
    ).signTransaction(tx, (err, signedTransaction) => {
        console.log(err, signedTransaction.rawTransaction)

        let called = false
        web3.eth.sendSignedTransaction(signedTransaction.rawTransaction)
            .on('error', cb)
            .on('transactionHash', (hash) => console.log('[\'' + '' + '\']: Staked ' + toWei('0.1', 'ether') + '. TxHash \'' + hash + '\'.'))
            .on('confirmation', () => {
                if (!called) {
                    called = true
                    cb()
                }
            })
    })

module.exports.contractCall = (tx, peerId, web3, cb) => {
    if (!tx.data)
        throw Error('Invalid contract call without a message invocation specified in the data field.')
        
    web3.eth.accounts.privateKeyToAccount(
        typeof peerId === 'string' ?
            peerId :
            '0x'.concat(peerId.privKey.marshal().toString('hex'))
    ).signTransaction(tx, (err, signedTransaction) => {
        console.log(err, signedTransaction.rawTransaction)

        web3.eth.sendSignedTransaction(signedTransaction.rawTransaction, cb)
    })
}

