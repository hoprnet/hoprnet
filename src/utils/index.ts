import { durations } from "@hoprnet/hopr-utils"

export * from "@hoprnet/hopr-utils"
export * from './crypto'
export * from './fs'
export * from './libp2p'
export * from './persistence'
export * from './concurrency'

const { promisify } = require('util')
import { promises as fsPromise, Stats, readFileSync } from 'fs'

// const { publicKeyConvert } = require('secp256k1')
const chalk = require('chalk')

// ==========================
// General methods
// ==========================

// module.exports.hash = (buf: Buffer) => {
//     return Buffer.from(sha3(buf).replace(/0x/, ''), 'hex')
// }
// /**
//  * Generate deep Copy of an instance
//  * @param {} instance instance of T
//  * @param {} Class T
//  */
// module.exports.deepCopy = (instance, Class) => {
//     if (typeof instance.toBuffer !== 'function' || !['function', 'number'].includes(typeof Class.SIZE) || typeof Class.fromBuffer !== 'function')
//         throw Error('Incompatible class and / or invalid instance.')

//     const buf = Buffer.alloc(Class.SIZE).fill(instance.toBuffer(), 0, Class.SIZE)

//     return Class.fromBuffer(buf)
// }

// module.exports.log = (peerId, msg) => console.log(`['${chalk.blue(peerId.toB58String())}']: ${msg}`)
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
// module.exports.bufferXOR = (buf1, buf2) => {
//     if (!Buffer.isBuffer(buf1) || !Buffer.isBuffer(buf2)) throw Error(`Input values have to be provided as Buffers. Got ${typeof buf1} and ${typeof buf2}`)

//     if (buf1.length !== buf2.length) throw Error(`Buffer must have the same length. Got buffers of length ${buf1.length} and ${buf2.length}.`)

//     const result = Buffer.alloc(buf1.length)

//     for (let i = 0; i < buf1.length; i = i + 1) {
//         result[i] = buf1[i] ^ buf2[i]
//     }
//     return result
// }

// module.exports.numberToBuffer = (i, length) => {
//     if (i < 0) throw Error('Not implemented!')

//     return Buffer.from(i.toString(16).padStart(length * 2, '0'), 'hex')
// }

// module.exports.bufferToNumber = buf => {
//     if (!Buffer.isBuffer(buf) || buf.length === 0) throw Error('Invalid input value. Expected a non-empty buffer.')

//     return parseInt(buf.toString('hex'), 16)
// }

// ==========================
// libp2p methods
// ==========================

// ==========================
// Ganache-core methods   <-- ONLY FOR TESTING
// ==========================
const ONE_MINUTE = durations.minutes(1)
/**
 * Mine a single block and increase the timestamp by the given amount.
 *
 * @notice The purpose of this method is to use it for testing with a local
 * testnet, i. e. Ganache.
 *
 * @param {Object} provider a valid Web3 provider
 * @param {Number} amountOfTime increase the timestamp by that amount of time, default 1 minute
 */
// module.exports.mineBlock = async (provider, amountOfTime = ONE_MINUTE) => {
//     const send = promisify(provider.send.bind(provider))

//     await send({
//         jsonrpc: '2.0',
//         method: 'evm_increaseTime',
//         params: [amountOfTime],
//         id: Date.now()
//     })

//     await send({
//         jsonrpc: '2.0',
//         method: 'evm_mine',
//         id: Date.now()
//     })

//     const { result } = await send({
//         jsonrpc: '2.0',
//         method: 'eth_blockNumber',
//         id: Date.now()
//     })

//     console.log(`\x1b[34mNow on block ${parseInt(result, 16)}.\x1b[0m`)
// }

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
// export async function compileIfNecessary(srcFiles: string[], artifacts: string[]): Promise<void> {
//     function findImports(path: string): { contents: string } {
//         return {
//             contents: readFileSync(`${process.cwd()}/node_modules/${path}`).toString()
//         }
//     }

//     async function compile() {
//         const sources: [string, string][] = await Promise.all<[string, string]>(
//             srcFiles.map((srcFile: string) => fsPromise.readFile(srcFile).then((file: Buffer) => [srcFile, file.toString()]))
//         )

//         const srcObject = {}
//         sources.forEach(([file, content]) => {
//             srcObject[file] = { content }
//         })

//         const input = {
//             language: 'Solidity',
//             sources: srcObject,
//             settings: {
//                 optimizer: {
//                     enabled: true,
//                     runs: 200
//                 },
//                 outputSelection: {
//                     '*': {
//                         '*': ['*']
//                     }
//                 }
//             }
//         }

//         const compiledContracts = JSON.parse(solc.compile(JSON.stringify(input), findImports))

//         if (compiledContracts.errors) {
//             throw compiledContracts.errors.map(err => err.formattedMessage).join('\n')
//         }

//         this.createDirectoryIfNotExists('build/contracts')

//         await Promise.all(
//             Object.entries(compiledContracts.contracts).map(array =>
//                 Promise.all(
//                     Object.entries(array[1]).map(([contractName, jsonObject]) =>
//                         fsPromise.writeFile(`${process.cwd()}/build/contracts/${contractName}.json`, JSON.stringify(jsonObject, null, '\t'))
//                     )
//                 )
//             )
//         )
//     }

//     try {
//         await Promise.all<void>(artifacts.map((artifact: string) => fsPromise.access(artifact)))
//     } catch (err) {
//         return compile()
//     }

//     const [srcTimes, artifactTimes] = await Promise.all<number[]>([
//         Promise.all<Stats>(srcFiles.map((srcFile: string) => fsPromise.stat(srcFile))).then((stats: Stats[]) => stats.map((stat: Stats) => stat.mtimeMs)),
//         Promise.all<Stats>(artifacts.map((artifact: string) => fsPromise.stat(artifact))).then((stats: Stats[]) => stats.map((stat: Stats) => stat.mtimeMs))
//     ])

//     if (Math.max(...srcTimes) > Math.min(...artifactTimes)) {
//         return compile()
//     }
// }

/**
 * Deploys the smart contract.
 *
 * @param index current index of the account of `FUNDING_PEER`
 * @param web3 instance of web3.js
 * @returns promise that resolve once the contract is compiled and deployed, otherwise
 * it rejects.
 */
// export async function deployContract(index, web3) {
//     const fundingPeer = await this.privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)

//     let compiledContract = await this.compileIfNecessary([`${process.cwd()}/contracts/HoprChannel.sol`], [`${process.cwd()}/build/contracts/HoprChannel.json`])

//     if (!compiledContract) compiledContract = require(`${process.cwd()}/build/contracts/HoprChannel.json`)

//     const receipt = await this.sendTransaction(
//         {
//             gas: 3000333,
//             gasPrice: process.env['GAS_PRICE'],
//             nonce: index,
//             data: `0x${compiledContract.evm.bytecode.object}`
//         },
//         fundingPeer,
//         web3
//     )

//     console.log(
//         `Deployed contract on ${chalk.magenta(process.env.NETWORK)} at ${chalk.green(receipt.contractAddress.toString('hex'))}\nNonce is now ${chalk.red(
//             index
//         )}.\n`
//     )

//     await updateContractAddress([`${process.cwd()}/.env`, `${process.cwd()}/.env.example`], receipt.contractAddress)
//     process.env.CONTRACT_ADDRESS = receipt.contractAddress

//     return receipt.contractAddress
// }

/**
 * Takes a contract address and changes every occurence of `CONTRACT_ADDRESS = //...` to
 * the given contract address
 * @param fileNames the files whose CONTRACT_ADDRESS should be changed
 * @param contractAddress the new contract address
 */
// function updateContractAddress(fileNames: string[], contractAddress: string): Promise<void[]> {
//     process.env[`CONTRACT_ADDRESS`] = contractAddress

//     return Promise.all<void>(
//         fileNames.map(async filename => {
//             let file: string = (await fsPromise.readFile(filename)).toString()
//             const regex = new RegExp(`CONTRACT_ADDRESS_${process.env.NETWORK.toUpperCase()}\\s{0,}=(\\s{0,}0x[0-9a-fA-F]{0,})?`, 'g')

//             file = file.replace(regex, `CONTRACT_ADDRESS_${process.env.NETWORK.toUpperCase()} = ${contractAddress}`)

//             await fsPromise.writeFile(filename, Buffer.from(file))
//         })
//     )
// }
