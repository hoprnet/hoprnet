'use strict'

const { sha3, toChecksumAddress } = require('web3-utils')
const { randomBytes } = require('crypto')
const { waterfall, parallel, map, some, each } = require('neo-async')
const { execFile } = require('child_process')
const fs = require('fs')
const libp2p_crypto = require('libp2p-crypto').keys
const PeerId = require('peer-id')
const rlp = require('rlp')
const PeerInfo = require('peer-info')
const Multiaddr = require('multiaddr')
const { publicKeyConvert } = require('secp256k1')
const scrypt = require('scrypt')
const chacha = require('chacha')
const read = require('read')

const COMPRESSED_PUBLIC_KEY_LENGTH = 33
const PRIVKEY_LENGTH = 32

// ==========================
// General methods
// ==========================

module.exports.hash = (buf) => {
    if (!Buffer.isBuffer(buf))
        throw Error('Invalid input. Please use a Buffer')

    return Buffer.from(sha3(buf).replace(/0x/, ''), 'hex')
}
/**
 * Generate deep Copy of an instance
 * @param  {} instance instance of T
 * @param  {} Class T
 */
module.exports.deepCopy = (instance, Class) => {
    if (typeof instance.toBuffer !== 'function' || !['function', 'number'].includes(typeof Class.SIZE) || typeof Class.fromBuffer !== 'function')
        throw Error('Incompatible class and / or invalid instance.')

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

module.exports.log = (peerId, msg) =>
    console.log(`['\x1b[34m${peerId.toB58String()}\x1b[0m']: ${msg}`)
// ==========================
// Buffer methods
// ==========================
/**
 * result = buf1 + buf2
 * @param  {Buffer} buf1
 * @param  {Buffer} buf2
 */
// module.exports.bufferADD = (buf1, buf2) => {
//     if (!Buffer.isBuffer(buf1))
//         throw Error('Expected a buffer. Got \"' + typeof buf1 + '\" instead.')

//     const a = Number.parseInt(buf1.toString('hex'))
//     let b, length

//     if (Buffer.isBuffer(buf2)) {
//         // Incorrect hex format ?
//         b = Number.parseInt(buf2.toString('hex'))
//         length = Math.max(buf1.length, buf2.length)

//     } else if (Number.isInteger(buf2)) {
//         b = buf2
//         length = buf1.length
//     } else {
//         throw Error('Invalid input values. Got \"' + typeof buf1 + '\" and \"' + typeof buf2 + '\".')
//     }

//     return module.exports.numberToBuffer(a + b, length)
// }
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
        throw Error(`Input values have to be provided as Buffers. Got ${typeof buf1} and ${typeof buf2}`)

    if (buf1.length !== buf2.length)
        throw Error(`Buffer must have the same length. Got buffers of length ${buf1.length} and ${buf2.length}.`)

    const result = Buffer.alloc(buf1.length)

    for (let i = 0; i < buf1.length; i = i + 1) {
        result[i] = buf1[i] ^ buf2[i]
    }
    return result
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
        throw Error(`Invalid input arguments. Please provide a positive subset size. Got '${subsetSize}' instead.`)

    if (!array || !Array.isArray(array))
        throw Error(`Invalid input parameters. Expected an Array. Got '${typeof array}' instead.`)

    if (subsetSize > array.length)
        throw Error('Invalid subset size. Subset size must not be greater than set size.')

    if (subsetSize == 0)
        return []

    if (subsetSize === array.length)
        // Returns a random permutation of all elements that pass
        // the test.
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
/**
 * Derives an Ethereum address from a given public key.
 * 
 * @param  {Buffer} pubKey given as compressed elliptic curve point.
 * 
 * @returns {String} e.g. 0xc1912fEE45d61C87Cc5EA59DaE31190FFFFf232d
 */
module.exports.pubKeyToEthereumAddress = (pubKey) => {
    if (!Buffer.isBuffer(pubKey) || pubKey.length !== COMPRESSED_PUBLIC_KEY_LENGTH)
        throw Error(`Invalid input parameter. Expected a Buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH}.`)

    const hash = sha3(publicKeyConvert(pubKey, false).slice(1))

    return toChecksumAddress(hash.replace(/(0x)[0-9a-fA-F]{24}([0-9a-fA-F]{20})/, '$1$2'))
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

        sender = Buffer.from(sender.replace(/0x/, ''), 'hex')
    }

    if (typeof otherParty === 'string') {
        if (otherParty.length !== 42) {
            throw Error('Invalid input parameters')
        }
        otherParty = Buffer.from(otherParty.replace(/0x/, ''), 'hex')
    }

    if (!Buffer.isBuffer(sender) || !Buffer.isBuffer(otherParty))
        throw Error('Invalid input parameters')

    if (sender.length != 20 || otherParty.length != 20)
        throw Error('Invalid input parameters')

    return Buffer.compare(sender, otherParty) < 0
}

const ETHEUREUM_ADDRESS_SIZE = 20 // Bytes

/**
 * Computes the ID that is used by the smart contract to 
 * store payment channels.
 * 
 * @param {String | Buffer} sender an ethereum address or the corresponding public key
 * @param {String | Buffer} counterparty another ethereum address or the corresponding public key
 * @returns {Buffer} the Id
 */
module.exports.getId = (sender, counterparty) => {
    if (Buffer.isBuffer(sender) && sender.length == COMPRESSED_PUBLIC_KEY_LENGTH) {
        sender = this.pubKeyToEthereumAddress(sender)
    }

    if (Buffer.isBuffer(counterparty) && counterparty.length == COMPRESSED_PUBLIC_KEY_LENGTH) {
        counterparty = this.pubKeyToEthereumAddress(counterparty)
    }

    if (typeof sender !== 'string' || typeof counterparty !== 'string')
        throw Error(`Invalid input parameters. Unable to convert ${typeof sender} and / or ${typeof counterparty} to an Ethereum address.`)

    sender = Buffer.from(sender.replace(/0x/, ''), 'hex')
    counterparty = Buffer.from(counterparty.replace(/0x/, ''), 'hex')

    if (module.exports.isPartyA(sender, counterparty)) {
        return module.exports.hash(Buffer.concat([sender, counterparty], 2 * ETHEUREUM_ADDRESS_SIZE))
    } else {
        return module.exports.hash(Buffer.concat([counterparty, sender], 2 * ETHEUREUM_ADDRESS_SIZE))
    }
}

// ==========================
// libp2p methods
// ==========================
/**
 * Converts a plain compressed ECDSA public key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * 
 * @notice Libp2p stores the keys in format that is derived from `protobuf`.
 * Using `libsecp256k1` directly does not work.
 * 
 * @param {Buffer | string} pubKey the plain public key
 * @param {function} cb callback called with `(err, peerId)`
 */
module.exports.pubKeyToPeerId = (pubKey, cb) => {
    if (typeof pubKey === 'string') {
        pubKey = Buffer.from(pubKey.replace(/0x/, ''), 'hex')
    }

    if (!Buffer.isBuffer(pubKey))
        return cb(Error(`Unable to parse public key to desired representation. Got ${pubKey.toString()}.`))

    if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
        return cb(Error(`Invalid public key. Expected a buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH} bytes. Got one of ${pubKey.length} bytes.`))

    return PeerId.createFromPubKey(new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PublicKey(pubKey).bytes, cb)
}

/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 * 
 * @param {Buffer | string} privKey the plain private key
 * @param {function} cb callback called with `(err, peerId)`
 */
module.exports.privKeyToPeerId = (privKey, cb) => {
    if (typeof privKey === 'string') {
        privKey = Buffer.from(privKey.replace(/0x/, ''), 'hex')
    }

    if (!Buffer.isBuffer(privKey))
        return cb(Error(`Unable to parse private key to desired representation. Got type '${typeof privKey}'.`))

    if (privKey.length != PRIVKEY_LENGTH)
        return cb(Error(`Invalid private key. Expected a buffer of size ${PRIVKEY_LENGTH} bytes. Got one of ${privKey.length} bytes.`))

    privKey = new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PrivateKey(privKey)

    return privKey.public.hash((err, hash) => {
        if (err)
            return cb(err)

        return cb(null, new PeerId(hash, privKey, privKey.public))
    })
}

// module.exports.privKeyToPeerId = (privKey, cb) =>
//     PeerId.createFromPrivKey(new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PrivateKey(privKey).bytes, cb)

// ==========================
// Ganache-core methods   <-- ONLY FOR TESTING
// ==========================
const ONE_MINUTE = 60 * 1000
/**
 * Mine a single block and increase the timestamp by the given amount.
 * 
 * @notice The purpose of this method is to use for testing with a local
 * testnet, i. e. Ganache.
 * 
 * @param {Object} provider a valid Web3 provider
 * @param {Number} amountOfTime increase the timestamp by that amount of time, default 1 minute
 */
module.exports.mineBlock = (provider, amountOfTime = ONE_MINUTE) => waterfall([
    (cb) => provider.send({
        jsonrpc: '2.0',
        method: 'evm_increaseTime',
        params: [amountOfTime],
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

        console.log(`\x1b[34mNow on block ${parseInt(response.result, 16)}.\x1b[0m`)
    })
])

// ==========================
// Web3.js methods
// ==========================
const { GAS_PRICE } = require('../constants')
/**
 * Creates a web3 account from a peerId instance.
 * 
 * @param {PeerId} peerId a peerId instance
 * @param {Web3} web3 a web3.js instance
 */
module.exports.peerIdToWeb3Account = (peerId, web3) => {
    if (!peerId.privKey)
        throw Error(`Unable to find private key. Please insert a peerId that is equipped with a private key.`)

    web3.eth.accounts.privateKeyToAccount('0x'.concat(peerId.privKey.marshal().toString('hex')))
}

/**
 * Signs a transaction with the private key that is given by 
 * the peerId instance and publishes it to the network given by
 * the web3.js instance
 * 
 * @param {Object} tx an Ethereum transaction
 * @param {Object} peerId a peerId
 * @param {Object} web3 a web3.js instance
 * @param {function} cb the function that is called when finished
 */
module.exports.sendTransaction = async (tx, peerId, web3, cb = () => { }) => {
    const signedTx = await this.peerIdToWeb3Account(peerId, web3).signTransaction(Object.assign(tx, {
        from: this.pubKeyToEthereumAddress(peerId.pubKey.marshal()),
        gasPrice: GAS_PRICE
    }))

    web3.eth.sendSignedTransaction(signedTx.rawTransaction)
        .on('error', cb)
        .on('receipt', (receipt) => {
            if (typeof receipt.status === 'string') {
                receipt.status = parseInt(receipt.status, 16)
            }

            if (typeof receipt.status === 'number') {
                receipt.status === Boolean(receipt.status)
            }

            if (!receipt.status)
                return cb(Error('Reverted tx.'))

            return cb(null, receipt)
        })
}

/**
 * Checks whether one of the src files is newer than one of
 * the artifacts.
 * 
 * @notice the method utilizes Truffle to compile the smart contracts.
 * Please make sure that Truffle is accessible by `npx`.
 * 
 * @param {Array} srcFiles the absolute paths of the source files
 * @param {Array} artifacts the absolute paths of the artifacts
 * @param {function} cb called afterward with `(err)`
 */
module.exports.compileIfNecessary = (srcFiles, artifacts, cb) => {
    function compile(cb) {
        console.log('Compiling smart contract ...')
        execFile('npx', ['truffle', 'compile'], (err, stdout, stderr) => {
            if (err) {
                cb(err)
            } else if (stderr) {
                console.log(`${stdout}\n\x1b[31m${stderr}\x1b[0m`)
            } else {
                console.log(stdout)
                cb()
            }
        })
    }

    waterfall([
        (cb) => some(artifacts, (file, cb) => fs.access(file, (err) => {
            cb(null, !err)
        }), cb),
        (filesExist, cb) => {
            if (!filesExist) {
                compile(cb)
            } else {
                parallel({
                    srcTime: (cb) => map(srcFiles, fs.stat, (err, stats) => {
                        if (err)
                            return cb(err)

                        return cb(null, stats.reduce((acc, current) => Math.max(acc, current.mtimeMs), 0))
                    }),
                    artifactTime: (cb) => map(artifacts, fs.stat, (err, stats) => {
                        if (err)
                            return cb(err)

                        return cb(null, stats.reduce((acc, current) => Math.min(acc, current.mtimeMs), Date.now()))
                    })
                }, (err, { srcTime, artifactTime }) => {
                    if (err)
                        return cb(err)

                    if (srcTime > artifactTime)
                        return compile(cb)

                    return cb()
                })
            }
        }
    ], cb)
}

/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 * 
 * @param {Buffer} serializePeerBook the encodes serialized peerBook
 * @param {PeerBook} peerBook a peerBook instance to store the peerInfo instances
 * @param {function} cb called when finished with `(err)`
 */
module.exports.deserializePeerBook = (serializedPeerBook, peerBook, cb) =>
    each(rlp.decode(serializedPeerBook), (serializedPeerInfo, cb) => {
        const peerId = PeerId.createFromBytes(serializedPeerInfo[0])

        if (serializedPeerInfo.length === 3) {
            peerId.pubKey = libp2p_crypto.unmarshalPublicKey(serializedPeerInfo[2])
        }

        PeerInfo.create(peerId, (err, peerInfo) => {
            if (err)
                return cb(err)

            serializedPeerInfo[1].forEach((multiaddr) => peerInfo.multiaddrs.add(Multiaddr(multiaddr)))
            peerBook.put(peerInfo)

            return cb()
        })
    }, cb)

/**
 * Serializes a given peerBook by serializing the included peerInfo instances.
 * 
 * @param {PeerBook} peerBook the peerBook instance
 * @returns the encoded peerBook
 */
module.exports.serializePeerBook = (peerBook) => {
    function serializePeerInfo(peerInfo) {
        const result = [
            peerInfo.id.toBytes(),
            peerInfo.multiaddrs.toArray().map(multiaddr => multiaddr.buffer)
        ]

        if (peerInfo.id.pubKey) {
            result.push(peerInfo.id.pubKey.bytes)
        }

        return result
    }

    const peerInfos = []
    peerBook.getAllArray().forEach(peerInfo => peerInfos.push(serializePeerInfo(peerInfo)))

    return rlp.encode(peerInfos)
}

const SALT_LENGTH = 32

/**
 * Serializes a given peerId by serializing the included private key and public key.
 * 
 * @param {PeerId} peerId the peerId that should be serialized
 * @param {function} cb called afterwards with `(err, encodedKeyPair)`
 */
module.exports.serializeKeyPair = (peerId, cb) => {
    const salt = randomBytes(SALT_LENGTH)
    const scryptParams = { N: 8192, r: 8, p: 16 }

    const question = 'Please type in the password that will be used to encrypt the generated key.'

    waterfall([
        (cb) => this.askForPassword(question, cb),
        (pw, isDefault, cb) => {
            console.log(`Done. Using peerId \x1b[34m${peerId.toB58String()}\x1b[0m\n`)

            const key = scrypt.hashSync(pw, scryptParams, 44, salt)

            const serializedPeerId = rlp.encode([
                peerId.toBytes(),
                peerId.privKey.bytes,
                peerId.pubKey.bytes
            ])

            const ciphertext = chacha
                .chacha20(key.slice(0, 32), key.slice(32, 32 + 12))
                .update(serializedPeerId)

            cb(null, rlp.encode([
                salt,
                ciphertext
            ]))
        }
    ], cb)
}

/**
 * Deserializes a serialized key pair and returns a peerId.
 * 
 * @notice This method will ask for a password to decrypt the encrypted
 * private key.
 * @notice The decryption of the private key makes use of a memory-hard
 * hash function and consumes therefore a lot of memory.
 * 
 * @param {Buffer} encryptedSerializedKeyPair the encoded and encrypted key pair
 * @param {function} cb called afterward with `(err, peerId)`
 */
module.exports.deserializeKeyPair = (encryptedSerializedKeyPair, cb) => {
    const encrypted = rlp.decode(encryptedSerializedKeyPair)

    const salt = encrypted[0]
    const ciphertext = encrypted[1]

    const scryptParams = { N: 8192, r: 8, p: 16 }

    const question = 'Please type in the password that was used to encrypt the key.'

    waterfall([
        (cb) => this.askForPassword(question, cb),
        (pw, isDefault, cb) => {
            const key = scrypt.hashSync(pw, scryptParams, 44, salt)

            const plaintext = chacha
                .chacha20(key.slice(0, 32), key.slice(32, 32 + 12))
                .update(ciphertext)

            const decoded = rlp.decode(plaintext)

            const peerId = PeerId.createFromBytes(decoded[0])

            libp2p_crypto.unmarshalPrivateKey(decoded[1], (err, privKey) => {
                if (err)
                    return cb(err)

                peerId.privKey = privKey
                peerId.pubKey = libp2p_crypto.unmarshalPublicKey(decoded[2])

                console.log(`Successfully restored ID \x1b[34m${peerId.toB58String()}\x1b[0m.`)

                return cb(null, peerId)
            })

        }
    ], cb)
}

const { DEBUG } = require('../constants')

/**
 * Asks the user for a password. Does not echo the password.
 * 
 * @param {string} question string that is displayed before the user input
 * @param {function} cb called afterwards with `(err, password)`
 */
module.exports.askForPassword = (question, cb) => {
    if (DEBUG) {
        console.log('Debug mode: using password Epo5kZTFidOCHrnL0MzsXNwN9St')
        cb(null, 'Epo5kZTFidOCHrnL0MzsXNwN9St', false)
    } else {
        read({
            prompt: question,
            silent: true,
            edit: true,
            replace: '*'
        }, cb)
    }
}

/**
 * Deletes recursively (and synchronously) all files in a directory.
 * 
 * @param {string} path the path to the directory
 */
module.exports.clearDirectory = (path) => {
    let files = [];
    if (fs.existsSync(path)) {
        files = fs.readdirSync(path);
        files.forEach(function (file, index) {
            const curPath = path + "/" + file;
            if (fs.lstatSync(curPath).isDirectory()) { // recurse
                deleteFolderRecursive(curPath);
            } else { // delete file
                fs.unlinkSync(curPath);
            }
        });
        fs.rmdirSync(path);
    }
}