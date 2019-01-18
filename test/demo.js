'use strict'

const { toChecksumAddress } = require('web3').utils
const { waterfall, timesSeries } = require('neo-async')
const { resolve } = require('path');

const { sendTransaction, privKeyToPeerId, log } = require('../src/utils')
const { createFundedNodes, compileIfNecessary } = require('./utils')

const { NET, GAS_PRICE, ROPSTEN_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const FUNDING_ACCOUNT = HARDCODED_ETH_ADDRESS
const FUNDING_KEY = HARDCODED_PRIV_KEY

const Ganache = require('ganache-core')

const Web3 = require('web3')

let web3
if (NET === 'ganache') {
    web3 = new Web3(Ganache.provider({
        accounts: [
            {
                balance: '0xd3c21bcecceda0000000',
                secretKey: FUNDING_KEY
            }
        ]
    }))
} else if (NET === 'ropsten') {
    web3 = new Web3(ROPSTEN_WSS_URL)
}

const AMOUUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 3

let index, compiledContract

console.log(
    'Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n' +
    'Please wait some time until the node is set up.\n' +
    '\x1b[2mThis may take some time ...\n' + 
    'Meanwhile you can start reading the wiki https://github.com/validitylabs/messagingProtocol/wiki\x1b[0m\n')

waterfall([
    (cb) => web3.eth.getTransactionCount(FUNDING_ACCOUNT, cb),
    (_index, cb) => {
        index = _index

        compileIfNecessary([resolve(__dirname, '../contracts/HoprChannel.sol')], [resolve(__dirname, '../build/contracts/HoprChannel.json')], cb)
    },
    (cb) => {
        compiledContract = require('../build/contracts/HoprChannel.json')

        if (NET === 'ganache') {
            sendTransaction({
                to: 0,
                gas: 3000333, // 2370333
                gasPrice: GAS_PRICE,
                nonce: index,
                data: compiledContract.bytecode
            }, privKeyToPeerId(FUNDING_KEY), web3, (err, receipt) => {
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
    (contractAddress, cb) => {
        const contract = new web3.eth.Contract(compiledContract.abi, toChecksumAddress(contractAddress))

        createFundedNodes(AMOUUNT_OF_NODES, contract, web3, privKeyToPeerId(FUNDING_KEY), index, cb)
    },
    (nodes, cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
        nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo.id)

        if (NET === 'ganache') {
            setTimeout(cb, 2000)
        } else {
            setTimeout(cb, 80 * 1000)
        }
    }, (err) => cb(err, nodes)),
    (nodes, cb) => nodes[1].paymentChannels.payout((err, result) => cb(err, nodes, result))
], (err, nodes, result) => {
    if (err)
        throw err

    log(nodes[1].peerInfo.id, `Finally received \x1b[35m\x1b[1m${result} wei\x1b[0m.`)

    process.exit(0)
})