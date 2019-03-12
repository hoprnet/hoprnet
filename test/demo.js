'use strict'

const { waterfall, timesSeries, times, map } = require('neo-async')
const { resolve } = require('path');

const { sendTransaction, privKeyToPeerId, log, compileIfNecessary } = require('../src/utils')
const { createFundedNodes } = require('./utils')
const rlp = require('rlp')

const { NETWORK, GAS_PRICE, INFURA_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const FUNDING_ACCOUNT = HARDCODED_ETH_ADDRESS
const FUNDING_KEY = HARDCODED_PRIV_KEY

const AMOUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 4


const Web3 = require('web3')
let provider, server, web3

console.log(
    'Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n' +
    'Please wait some time until the node is set up.\n' +
    '\x1b[2mThis may take some time ...\n' +
    'Meanwhile you can start reading the wiki at https://github.com/validitylabs/messagingProtocol/wiki\x1b[0m\n')

let index, compiledContract, fundingPeer
waterfall([
    (cb) => {
        if (NETWORK === 'ropsten') {
            provider = INFURA_WSS_URL

            return cb()
        } else if (NETWORK === 'ganache') {
            const GANACHE_PORT = 8545
            const GANACHE_HOSTNAME = 'localhost'
            server = require('ganache-core').server({
                accounts: [
                    {
                        balance: '0xd3c21bcecceda0000000',
                        secretKey: FUNDING_KEY
                    }
                ]
            })
            server.listen(GANACHE_PORT, GANACHE_HOSTNAME, (err) => {
                if (err)
                    return cb(err)

                console.log(`Successfully started local Ganache instance at 'ws://${GANACHE_HOSTNAME}:${GANACHE_PORT}'.`)

                provider = `ws://${GANACHE_HOSTNAME}:${GANACHE_PORT}`

                return setImmediate(cb)
            })
        }
    },
    (cb) => privKeyToPeerId(FUNDING_KEY, cb),
    (_fundingPeer, cb) => {
        web3 = new Web3(provider)
        fundingPeer = _fundingPeer
        web3.eth.getTransactionCount(FUNDING_ACCOUNT, cb)
    },
    (_index, cb) => {
        index = _index

        return compileIfNecessary([resolve(__dirname, '../contracts/HoprChannel.sol')], [resolve(__dirname, '../build/contracts/HoprChannel.json')], cb)
    },
    (_, cb) => {
        compiledContract = require('../build/contracts/HoprChannel.json')

        if (NETWORK === 'ganache') {
            sendTransaction({
                to: 0,
                gas: 3000333, // 2370333
                gasPrice: GAS_PRICE,
                nonce: index,
                data: compiledContract.bytecode
            }, fundingPeer, web3, (err, receipt) => {
                if (err)
                    throw err

                index = index + 1

                console.log(`\nDeployed contract at \x1b[32m${receipt.contractAddress}\x1b[0m.\nNonce is now \x1b[31m${index}\x1b[0m.\n`)

                return cb(null, receipt.contractAddress)
            })
        } else {
            return cb(null, CONTRACT_ADDRESS)
        }
    },
    (contractAddress, cb) => createFundedNodes(AMOUNT_OF_NODES, {
        provider: provider,
        contractAddress: contractAddress
    }, fundingPeer, index, cb),
    (nodes, cb) => waterfall([
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[3].peerInfo.id)

            if (NETWORK === 'ganache') {
                setTimeout(cb, 2000)
            } else {
                setTimeout(cb, 60 * 1000)
            }
        }, cb),
        (_, cb) => map(nodes.slice(0, 1), (node, cb) => node.paymentChannels.closeChannels(cb), cb),
        (results, cb) => times(1, (n, cb) => {
            log(nodes[n].peerInfo.id, `Finally ${results[0].isNeg() ? 'spent' : 'received'} \x1b[35m\x1b[1m${results[n].abs()} wei\x1b[0m.`)
            nodes[n].stop(cb)
        }, cb)
    ], (err, _) => cb(err)),
    (cb) => {
        if (NETWORK === 'ganache') {
            setTimeout(server.close, 2000, cb)
        } else {
            return cb()
        }
    }
], (err) => {
    if (err)
        throw err
})