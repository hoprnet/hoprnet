'use strict'

const { sha3, toChecksumAddress } = require('web3-utils')
const { promisify } = require('util')
const fs = require('fs')
import { promises as fsPromise, Stats, readFileSync } from 'fs'

const { publicKeyConvert } = require('secp256k1')
const solc = require('solc')
const chalk = require('chalk')



export * from './u8a'

// ==========================
// General methods
// ==========================

module.exports.hash = (buf: Buffer) => {
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

module.exports.log = (peerId, msg) => console.log(`['${chalk.blue(peerId.toB58String())}']: ${msg}`)
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
        throw Error(
            `Invalid input parameter. Expected a Buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${
                pubKey.length ? ` of length ${pubKey.length}` : ''
            }.`
        )

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

    const { result } = await send({
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
module.exports.sendTransaction = async (tx, peerId, web3) => {
    const signedTransaction = await this.signTransaction(tx, peerId, web3)

    return web3.eth.sendSignedTransaction(signedTransaction.rawTransaction).then(receipt => {
        if (typeof receipt.status === 'string') {
            receipt.status = parseInt(receipt.status, 16)
        }

        if (typeof receipt.status === 'number') {
            receipt.status === Boolean(receipt.status)
        }

        if (!receipt.status) throw Error('Reverted tx.')

        return receipt
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
 */
export async function compileIfNecessary(srcFiles: string[], artifacts: string[]): Promise<void> {
    function findImports(path: string): { contents: string } {
        return {
            contents: readFileSync(`${process.cwd()}/node_modules/${path}`).toString()
        }
    }

    async function compile() {
        const sources: [string, string][] = await Promise.all<[string, string]>(
            srcFiles.map((srcFile: string) => fsPromise.readFile(srcFile).then((file: Buffer) => [srcFile, file.toString()]))
        )

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

        if (compiledContracts.errors) {
            throw compiledContracts.errors.map(err => err.formattedMessage).join('\n')
        }

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
        await Promise.all<void>(artifacts.map((artifact: string) => fsPromise.access(artifact)))
    } catch (err) {
        return compile()
    }

    const [srcTimes, artifactTimes] = await Promise.all<number[]>([
        Promise.all<Stats>(srcFiles.map((srcFile: string) => fsPromise.stat(srcFile))).then((stats: Stats[]) => stats.map((stat: Stats) => stat.mtimeMs)),
        Promise.all<Stats>(artifacts.map((artifact: string) => fsPromise.stat(artifact))).then((stats: Stats[]) => stats.map((stat: Stats) => stat.mtimeMs))
    ])

    if (Math.max(...srcTimes) > Math.min(...artifactTimes)) {
        return compile()
    }
}

/**
 * Deploys the smart contract.
 *
 * @param index current index of the account of `FUNDING_PEER`
 * @param web3 instance of web3.js
 * @returns promise that resolve once the contract is compiled and deployed, otherwise
 * it rejects.
 */
export async function deployContract(index, web3) {
    const fundingPeer = await this.privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)

    let compiledContract = await this.compileIfNecessary([`${process.cwd()}/contracts/HoprChannel.sol`], [`${process.cwd()}/build/contracts/HoprChannel.json`])

    if (!compiledContract) compiledContract = require(`${process.cwd()}/build/contracts/HoprChannel.json`)

    const receipt = await this.sendTransaction(
        {
            gas: 3000333,
            gasPrice: process.env['GAS_PRICE'],
            nonce: index,
            data: `0x${compiledContract.evm.bytecode.object}`
        },
        fundingPeer,
        web3
    )

    console.log(
        `Deployed contract on ${chalk.magenta(process.env.NETWORK)} at ${chalk.green(receipt.contractAddress.toString('hex'))}\nNonce is now ${chalk.red(
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
 * @param fileNames the files whose CONTRACT_ADDRESS should be changed
 * @param contractAddress the new contract address
 */
function updateContractAddress(fileNames: string[], contractAddress: string): Promise<void[]> {
    process.env[`CONTRACT_ADDRESS`] = contractAddress

    return Promise.all<void>(
        fileNames.map(async filename => {
            let file: string = (await fsPromise.readFile(filename)).toString()
            const regex = new RegExp(`CONTRACT_ADDRESS_${process.env.NETWORK.toUpperCase()}\\s{0,}=(\\s{0,}0x[0-9a-fA-F]{0,})?`, 'g')

            file = file.replace(regex, `CONTRACT_ADDRESS_${process.env.NETWORK.toUpperCase()} = ${contractAddress}`)

            await fsPromise.writeFile(filename, Buffer.from(file))
        })
    )
}