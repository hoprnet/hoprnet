'use strict'

const { waterfall, timesSeries, series, each } = require('neo-async')
const { resolve } = require('path');

const { sendTransaction, privKeyToPeerId, log, compileIfNecessary } = require('../src/utils')
const { createFundedNodes } = require('./utils')

const { NET, GAS_PRICE, ROPSTEN_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const FUNDING_ACCOUNT = HARDCODED_ETH_ADDRESS
const FUNDING_KEY = HARDCODED_PRIV_KEY

const AMOUUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 3

let index, compiledContract

let provider
if (NET === 'ropsten') {
    provider = ROPSTEN_WSS_URL
} else if (NET === 'ganache') {
    const GANACHE_PORT = 8545
    const GANACHE_HOSTNAME = 'localhost'
    const server = require('ganache-core').server({
        accounts: [
            {
                balance: '0xd3c21bcecceda0000000',
                secretKey: FUNDING_KEY
            }
        ]
    })
    server.listen(GANACHE_PORT, GANACHE_HOSTNAME)

    provider = `ws://${GANACHE_HOSTNAME}:${GANACHE_PORT}`

    console.log(`Successfully started local Ganache instance at 'ws://${GANACHE_HOSTNAME}:${GANACHE_PORT}'.`)
}
const Web3 = require('Web3')
const web3 = new Web3(provider)

const fundingPeer = privKeyToPeerId(FUNDING_KEY)

console.log(
    'Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n' +
    'Please wait some time until the node is set up.\n' +
    '\x1b[2mThis may take some time ...\n' +
    'Meanwhile you can start reading the wiki at https://github.com/validitylabs/messagingProtocol/wiki\x1b[0m\n')

waterfall([
    (cb) => web3.eth.getTransactionCount(FUNDING_ACCOUNT, cb),
    (_index, cb) => {
        index = _index

        compileIfNecessary([resolve(__dirname, '../contracts/HoprChannel.sol')], [resolve(__dirname, '../build/contracts/HoprChannel.json')], cb)
    },
    (_, cb) => {
        compiledContract = require('../build/contracts/HoprChannel.json')

        if (NET === 'ganache') {
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

                cb(null, receipt.contractAddress)
            })
        } else {
            cb(null, CONTRACT_ADDRESS)
        }
    },
    (contractAddress, cb) => createFundedNodes(AMOUUNT_OF_NODES, {
        provider: provider,
        contractAddress: contractAddress
    }, fundingPeer, index, cb),
    (nodes, cb) => series([
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage('Psst ... secret message from Validity Labs!@' + Date.now().toString(), nodes[3].peerInfo.id)
    
            if (NET === 'ganache') {
                setTimeout(cb, 2000)
            } else {
                setTimeout(cb, 80 * 1000)
            }
        }, cb),
        (cb) => each(nodes, (node, cb) => {
            node.paymentChannels.closeChannels((err, result) => {
                log(node.peerInfo.id, `Finally ${result.isNeg() ? 'spent' : 'received'} \x1b[35m\x1b[1m${result.abs()} wei\x1b[0m.`)
                node.stop(cb)
            })
        }, cb)
    ], cb)
], () => {})