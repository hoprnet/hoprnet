'use strict'

const { sha3, toChecksumAddress } = require('web3-utils')
const { randomBytes } = require('crypto')
const { promisify } = require('util')
const fs = require('fs')
const fsPromise = require('fs').promises
const libp2p_crypto = require('libp2p-crypto').keys
const PeerId = require('peer-id')
const rlp = require('rlp')
const PeerInfo = require('peer-info')
const Multiaddr = require('multiaddr')
const Multihash = require('multihashes')
const { publicKeyConvert } = require('secp256k1')
const scrypt = require('scrypt')
const chacha = require('chacha')
const read = require('read')
const solc = require('solc')
const chalk = require('chalk')
const crypto = require('crypto')

const COMPRESSED_PUBLIC_KEY_LENGTH = 33
const PRIVKEY_LENGTH = 32

// ==========================
// General methods
// ==========================

module.exports.hash = buf => {
    if (!Buffer.isBuffer(buf)) throw Error('Invalid input. Please use a Buffer')

    return Buffer.from(sha3(buf).replace(/0x/, ''), 'hex')
}
/**
 * Generate deep Copy of an instance
 * @param {} instance instance of T
 * @param {} Class T
 */
module.exports.deepCopy = (instance, Class) => {
    if (typeof instance.toBuffer !== 'function' || !['function', 'number'].includes(typeof Class.SIZE) || typeof Class.fromBuffer !== 'function')
        throw Error('Incompatible class and / or invalid instance.')

    const buf = Buffer.alloc(Class.SIZE).fill(instance.toBuffer(), 0, Class.SIZE)

    return Class.fromBuffer(buf)
}

/**
 * Parse JSON while recovering all Buffer elements
 * @param  {String} str JSON string
 */
module.exports.parseJSON = str =>
    JSON.parse(str || '', (key, value) => {
        if (value && value.type === 'Buffer') {
            return Buffer.from(value.data)
        }

        return value
    })

module.exports.log = (peerId, msg) => console.log(`['\x1b[34m${peerId.toB58String()}\x1b[0m']: ${msg}`)
// ==========================
// Buffer methods
// ==========================
/**
 * Bitwise XOR of two Buffers.
 *
 * @param  {Buffer} buf1 first Buffer
 * @param  {Buffer} buf2 second Buffer
 *
 * @returns {Buffer} @param buf1 ^ @param buf2
 */
module.exports.bufferXOR = (buf1, buf2) => {
    if (!Buffer.isBuffer(buf1) || !Buffer.isBuffer(buf2)) throw Error(`Input values have to be provided as Buffers. Got ${typeof buf1} and ${typeof buf2}`)

    if (buf1.length !== buf2.length) throw Error(`Buffer must have the same length. Got buffers of length ${buf1.length} and ${buf2.length}.`)

    const result = Buffer.alloc(buf1.length)

    for (let i = 0; i < buf1.length; i = i + 1) {
        result[i] = buf1[i] ^ buf2[i]
    }
    return result
}

module.exports.numberToBuffer = (i, length) => {
    if (i < 0) throw Error('Not implemented!')

    return Buffer.from(i.toString(16).padStart(length * 2, '0'), 'hex')
}

module.exports.bufferToNumber = buf => {
    if (!Buffer.isBuffer(buf) || buf.length === 0) throw Error('Invalid input value. Expected a non-empty buffer.')

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
 * @param  {Function} filter called with `(peerInfo)` and should return `true`
 * for every node that should be in the subset
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

    if (!array || !Array.isArray(array)) throw Error(`Invalid input parameters. Expected an Array. Got '${typeof array}' instead.`)

    if (subsetSize > array.length) throw Error('Invalid subset size. Subset size must not be greater than set size.')

    if (subsetSize <= 0) return []

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
    let found,
        breakUp = false

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
module.exports.randomPermutation = array => {
    if (!Array.isArray(array)) throw Error("Invalid input parameters. Got '" + typeof array + "' instead of Buffer.")

    if (array.length <= 1) return array

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

/**
 * @param {Number} start
 * @param {Number} end
 * @returns {Number} random number between @param start and @param end
 */
module.exports.randomNumber = (start, end) => {
    if (!end) {
        end = start
        start = 0
    }

    if (start >= end) throw Error('Invalid interval.')

    if (start + 1 == end) return start

    const byteAmount = Math.max(Math.ceil(Math.log2(end - start)) / 8, 1)

    return start + (this.bufferToNumber(randomBytes(byteAmount)) % end)
}

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
module.exports.pubKeyToEthereumAddress = pubKey => {
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
        if (sender.length !== 42) throw Error('Invalid input parameters')

        sender = Buffer.from(sender.replace(/0x/, ''), 'hex')
    }

    if (typeof otherParty === 'string') {
        if (otherParty.length !== 42) {
            throw Error('Invalid input parameters')
        }
        otherParty = Buffer.from(otherParty.replace(/0x/, ''), 'hex')
    }

    if (!Buffer.isBuffer(sender) || !Buffer.isBuffer(otherParty)) throw Error('Invalid input parameters')

    if (sender.length != 20 || otherParty.length != 20) throw Error('Invalid input parameters')

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
 * @returns {Promise<PeerId>}
 */
module.exports.pubKeyToPeerId = (pubKey) => {
    if (typeof pubKey === 'string') {
        pubKey = Buffer.from(pubKey.replace(/0x/, ''), 'hex')
    }

    if (!Buffer.isBuffer(pubKey)) return cb(Error(`Unable to parse public key to desired representation. Got ${pubKey.toString()}.`))

    if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
        return cb(Error(`Invalid public key. Expected a buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH} bytes. Got one of ${pubKey.length} bytes.`))

    pubKey = new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PublicKey(pubKey)

    return PeerId.createFromPubKey(pubKey.bytes)
}

/**
 * Converts a plain compressed ECDSA private key over the curve `secp256k1`
 * to a peerId in order to use it with libp2p.
 * It equips the generated peerId with private key and public key.
 *
 * @param {Buffer | string} privKey the plain private key
 */
module.exports.privKeyToPeerId = (privKey, cb) => {
    if (cb) throw Error('TODO -> promisify')

    if (typeof privKey === 'string') privKey = Buffer.from(privKey.replace(/0x/, ''), 'hex')

    if (!Buffer.isBuffer(privKey)) return cb(Error(`Unable to parse private key to desired representation. Got type '${typeof privKey}'.`))

    if (privKey.length != PRIVKEY_LENGTH)
        return cb(Error(`Invalid private key. Expected a buffer of size ${PRIVKEY_LENGTH} bytes. Got one of ${privKey.length} bytes.`))

    privKey = new libp2p_crypto.supportedKeys.secp256k1.Secp256k1PrivateKey(privKey)

    return PeerId.createFromPrivKey(privKey.bytes)
}

/**
 * Takes a peerId and returns a peerId with the public key set to the corresponding
 * public key.
 *
 * @param {PeerId} peerId the PeerId instance that has probably no pubKey set
 */
module.exports.addPubKey = async peerId => {
    if (PeerId.isPeerId(peerId) && peerId.pubKey) return peerId

    peerId.pubKey = await libp2p_crypto.unmarshalPublicKey(Multihash.decode(peerId.toBytes()).digest)

    return peerId
}

// ==========================
// Ganache-core methods   <-- ONLY FOR TESTING
// ==========================
const ONE_MINUTE = 60 * 1000
/**
 * Mine a single block and increase the timestamp by the given amount.
 *
 * @notice The purpose of this method is to use it for testing with a local
 * testnet, i. e. Ganache.
 *
 * @param {Object} provider a valid Web3 provider
 * @param {Number} amountOfTime increase the timestamp by that amount of time, default 1 minute
 */
module.exports.mineBlock = async (provider, amountOfTime = ONE_MINUTE) => {
    const send = promisify(provider.send.bind(provider))

    await send({
        jsonrpc: '2.0',
        method: 'evm_increaseTime',
        params: [amountOfTime],
        id: Date.now()
    })

    await send({
        jsonrpc: '2.0',
        method: 'evm_mine',
        id: Date.now()
    })

    const {result} = await send({
        jsonrpc: '2.0',
        method: 'eth_blockNumber',
        id: Date.now()
    })

    console.log(`\x1b[34mNow on block ${parseInt(result, 16)}.\x1b[0m`)
}

// ==========================
// Web3.js methods
// ==========================
/**
 * Creates a web3 account from a peerId instance.
 *
 * @param {PeerId} peerId a peerId instance
 * @param {Web3} web3 a web3.js instance
 */
module.exports.peerIdToWeb3Account = (peerId, web3) => {
    if (!peerId.privKey) throw Error(`Unable to find private key. Please insert a peerId that is equipped with a private key.`)

    return web3.eth.accounts.privateKeyToAccount('0x'.concat(peerId.privKey.marshal().toString('hex')))
}

module.exports.signTransaction = async (tx, peerId, web3) => {
    const account = this.peerIdToWeb3Account(peerId, web3)

    return account.signTransaction(
        Object.assign(tx, {
            from: this.pubKeyToEthereumAddress(peerId.pubKey.marshal()),
            gasPrice: await web3.eth.getGasPrice()
        })
    )
}

/**
 * Signs a transaction with the private key that is given by
 * the peerId instance and publishes it to the network given by
 * the web3.js instance
 *
 * @param {Object} tx an Ethereum transaction
 * @param {Object} peerId a peerId
 * @param {Object} web3 a web3.js instance
 */
module.exports.sendTransaction = async (tx, peerId, web3) =>
    web3.eth.sendSignedTransaction((await this.signTransaction(tx, peerId, web3)).rawTransaction).then(receipt => {
        if (typeof receipt.status === 'string') {
            receipt.status = parseInt(receipt.status, 16)
        }

        if (typeof receipt.status === 'number') {
            receipt.status === Boolean(receipt.status)
        }

        if (!receipt.status) return cb(Error('Reverted tx.'))

        return receipt
    })

/**
 * Checks whether one of the src files is newer than one of
 * the artifacts.
 *
 * @notice the method utilizes Truffle to compile the smart contracts.
 * Please make sure that Truffle is accessible by `npx`.
 *
 * @param {Array} srcFiles the absolute paths of the source files
 * @param {Array} artifacts the absolute paths of the artifacts
 */
module.exports.compileIfNecessary = async (srcFiles, artifacts) => {
    function findImports(path) {
        return {
            contents: fs.readFileSync(`${process.cwd()}/node_modules/${path}`).toString()
        }
    }

    const compile = async () => {
        const sources = await Promise.all(srcFiles.map(srcFile => fsPromise.readFile(srcFile).then(file => [srcFile, file.toString()])))

        const srcObject = {}
        sources.forEach(([file, content]) => {
            srcObject[file] = { content }
        })

        const input = {
            language: 'Solidity',
            sources: srcObject,
            settings: {
                optimizer: {
                    enabled: true,
                    runs: 200
                },
                outputSelection: {
                    '*': {
                        '*': ['*']
                    }
                }
            }
        }
        const compiledContracts = JSON.parse(solc.compile(JSON.stringify(input), findImports))

        if (compiledContracts.errors) throw compiledContracts.errors.map(err => err.formattedMessage).join('\n')

        this.createDirectoryIfNotExists('build/contracts')

        await Promise.all(
            Object.entries(compiledContracts.contracts).map(array =>
                Promise.all(
                    Object.entries(array[1]).map(([contractName, jsonObject]) =>
                        fsPromise.writeFile(`${process.cwd()}/build/contracts/${contractName}.json`, JSON.stringify(jsonObject, null, '\t'))
                    )
                )
            )
        )
    }

    try {
        await Promise.all(artifacts.map(artifact => fsPromise.access(artifact)))
    } catch (err) {
        return compile()
    }

    const srcTimes = (await Promise.all(srcFiles.map(srcFile => fsPromise.stat(srcFile)))).map(stat => stat.mtimeMs)
    const artifactTimes = (await Promise.all(artifacts.map(artifact => fsPromise.stat(artifact)))).map(stat => stat.mtimeMs)

    if (Math.max(srcTimes) > Math.min(artifactTimes)) {
        return compile()
    }
}

/**
 * Deploys the smart contract.
 *
 * @param {Number} index current index of the account of `FUNDING_PEER`
 * @param {Web3} web3 instance of web3.js
 * @returns {Promise} promise that resolve once the contract is compiled and deployed, otherwise
 * it rejects.
 */
module.exports.deployContract = async (index, web3) => {
    const fundingPeer = await this.privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)

    let compiledContract = await this.compileIfNecessary([`${process.cwd()}/contracts/HoprChannel.sol`], [`${process.cwd()}/build/contracts/HoprChannel.json`])

    if (!compiledContract) compiledContract = require(`${process.cwd()}/build/contracts/HoprChannel.json`)

    const receipt = await this.sendTransaction(
        {
            gas: 3000333, // 2370333
            gasPrice: process.env.GAS_PRICE,
            nonce: index,
            data: '0x'.concat(compiledContract.evm.bytecode.object)
        },
        fundingPeer,
        web3
    )

    console.log(
        `Deployed contract on ${chalk.magenta(process.env.NETWORK)} at ${chalk.green(receipt.contractAddress.toString('hex'))}.\nNonce is now ${chalk.red(
            index
        )}.\n`
    )

    await updateContractAddress([`${process.cwd()}/.env`, `${process.cwd()}/.env.example`], receipt.contractAddress)
    process.env.CONTRACT_ADDRESS = receipt.contractAddress

    return receipt.contractAddress
}

/**
 * Takes a contract address and changes every occurence of `CONTRACT_ADDRESS = //...` to
 * the given contract address
 * @param {string[]} fileNames the files whose CONTRACT_ADDRESS should be changed
 * @param {string} contractAddress the new contract address
 */
function updateContractAddress(fileNames, contractAddress) {
    if (!Array.isArray(fileNames)) fileNames = [fileNames]

    process.env[`CONTRACT_ADDRESS`] = contractAddress

    return Promise.all(
        fileNames.map(async filename => {
            let file = (await fsPromise.readFile(filename)).toString()
            const regex = new RegExp(`CONTRACT_ADDRESS_${process.env.NETWORK.toUpperCase()}\\s{0,}=(\\s{0,}0x[0-9a-fA-F]{0,})?`, 'g')

            file = file.replace(regex, `CONTRACT_ADDRESS_${process.env.NETWORK.toUpperCase()} = ${contractAddress}`)

            await fsPromise.writeFile(filename, Buffer.from(file))
        })
    )
}
/**
 * Decodes the serialized peerBook and inserts the peerInfos in the given
 * peerBook instance.
 *
 * @param {Buffer} serializePeerBook the encodes serialized peerBook
 * @param {PeerBook} peerBook a peerBook instance to store the peerInfo instances
 */
module.exports.deserializePeerBook = async (serializedPeerBook, peerBook) => {
    if (!serializedPeerBook) return peerBook

    const serializedPeerInfos = rlp.decode(serializedPeerBook)

    await Promise.all(
        serializedPeerInfos.map(async serializedPeerInfo => {
            const peerId = PeerId.createFromBytes(serializedPeerInfo[0])

            if (serializedPeerInfo.length === 3) {
                peerId.pubKey = libp2p_crypto.unmarshalPublicKey(serializedPeerInfo[2])
            }

            const peerInfo = await PeerInfo.create(peerId)
            serializedPeerInfo[1].forEach(multiaddr => peerInfo.multiaddrs.add(Multiaddr(multiaddr)))
            peerBook.put(peerInfo)
        })
    )

    return peerBook
}

/**
 * Serializes a given peerBook by serializing the included peerInfo instances.
 *
 * @param {PeerBook} peerBook the peerBook instance
 * @returns the encoded peerBook
 */
module.exports.serializePeerBook = peerBook => {
    function serializePeerInfo(peerInfo) {
        const result = [peerInfo.id.toBytes(), peerInfo.multiaddrs.toArray().map(multiaddr => multiaddr.buffer)]

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
module.exports.serializeKeyPair = async peerId => {
    const salt = randomBytes(SALT_LENGTH)
    const scryptParams = { N: 8192, r: 8, p: 16 }

    const question = 'Please type in the password that will be used to encrypt the generated key.'

    const pw = await this.askForPassword(question)

    console.log(`Done. Using peerId \x1b[34m${peerId.toB58String()}\x1b[0m\n`)

    const key = scrypt.hashSync(pw, scryptParams, 32, salt)
    const iv = crypto.randomBytes(12)

    const serializedPeerId = Buffer.concat([Buffer.alloc(16, 0), peerId.marshal()])

    const ciphertext = chacha.chacha20(key, iv).update(serializedPeerId)

    return rlp.encode([salt, iv, ciphertext])
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
module.exports.deserializeKeyPair = async encryptedSerializedKeyPair => {
    const [salt, iv, ciphertext] = rlp.decode(encryptedSerializedKeyPair)

    const scryptParams = { N: 8192, r: 8, p: 16 }

    const question = 'Please type in the password that was used to encrypt the key.'

    let plaintext
    do {
        const pw = await this.askForPassword(question)

        const key = scrypt.hashSync(pw, scryptParams, 32, salt)

        plaintext = chacha.chacha20(key, iv).update(ciphertext)
    } while (!plaintext.slice(0, 16).equals(Buffer.alloc(16, 0)))

    const peerId = await PeerId.createFromProtobuf(plaintext)
    console.log(`Successfully restored ID \x1b[34m${peerId.toB58String()}\x1b[0m.`)

    return peerId
}

/**
 * Asks the user for a password. Does not echo the password.
 *
 * @param {string} question string that is displayed before the user input
 * @param {function} cb called afterwards with `(err, password)`
 */
module.exports.askForPassword = (question, cb) =>
    new Promise((resolve, reject) => {
        if (process.env.DEBUG === 'true') {
            console.log('Debug mode: using password Epo5kZTFidOCHrnL0MzsXNwN9St')
            resolve('Epo5kZTFidOCHrnL0MzsXNwN9St')
        } else {
            read(
                {
                    prompt: question,
                    silent: true,
                    edit: true,
                    replace: '*'
                },
                (err, pw, isDefault) => {
                    if (err) return reject(err)

                    resolve(pw)
                }
            )
        }
    })

/**
 * Deletes recursively (and synchronously) all files in a directory.
 *
 * @param {string} path the path to the directory
 */
module.exports.clearDirectory = path => {
    let files = []
    if (fs.existsSync(path)) {
        files = fs.readdirSync(path)
        files.forEach(function(file, index) {
            const curPath = path + '/' + file
            if (fs.lstatSync(curPath).isDirectory()) {
                // recurse
                deleteFolderRecursive(curPath)
            } else {
                // delete file
                fs.unlinkSync(curPath)
            }
        })
        fs.rmdirSync(path)
    }
}

/**
 * Creates a directory if it doesn't exist.
 *
 * @example
 * createDirectoryIfNotExists('db/testnet') // creates `./db` and `./db/testnet`
 * @param {string} path
 */
module.exports.createDirectoryIfNotExists = path => {
    const chunks = path.split('/')

    chunks.reduce((searchPath, chunk) => {
        searchPath += '/'
        searchPath += chunk
        try {
            fs.accessSync(`${process.cwd()}${searchPath}`)
        } catch (err) {
            fs.mkdirSync(`${process.cwd()}${searchPath}`)
        }
        return searchPath
    }, '')
}
