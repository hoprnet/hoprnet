'use strict'

const { readFileSync } = require('fs')
const { toWei, toChecksumAddress } = require('web3').utils
const { waterfall, times, timesSeries, each } = require('neo-async')
const { resolve } = require('path');


const { pubKeyToEthereumAddress, sendTransaction, privKeyToPeerId, log } = require('../src/utils')
const { warmUpNodes } = require('./utils')

const { NET, GAS_PRICE, STAKE_GAS_AMOUNT, ROPSTEN_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const { createNode } = require('../src')
const getContract = require('../contracts')

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
const AMOUNT_OF_MESSAGES = 4

let index

waterfall([
    (cb) => web3.eth.getTransactionCount(FUNDING_ACCOUNT, cb),
    (_index, cb) => {
        index = _index

        if (NET === 'ganache') {
            getContract((err, compiledContract) => sendTransaction({
                to: 0,
                gas: 3000333, // 2370333
                gasPrice: GAS_PRICE,
                nonce: index,
                data: '0x'.concat(compiledContract.binary.toString())
            }, privKeyToPeerId(FUNDING_KEY), web3, (err, receipt) => {
                if (err)
                    throw err
                    
                index = index + 1

                console.log(`\nDeployed contract at \x1b[32m${receipt.contractAddress}\x1b[0m.\nNonce is now \x1b[31m${index}\x1b[0m.\n`)

                cb(null, receipt.contractAddress)
            }))
        } else {
            cb(null, CONTRACT_ADDRESS)
        }
    },
    (contractAddress, cb) => {
        const contract = new web3.eth.Contract(JSON.parse(readFileSync(resolve(__dirname, '../contracts/HoprChannel.abi'))), toChecksumAddress(contractAddress))

        times(AMOUUNT_OF_NODES, (_, cb) =>
            createNode({
                contract: contract,
                web3: web3
            }, cb), cb)
    },
    (nodes, cb) => warmUpNodes(nodes, cb),
    (nodes, cb) => times(AMOUUNT_OF_NODES, (n, cb) =>
        sendTransaction({
            to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
            value: toWei('0.05', 'ether'),
            gas: STAKE_GAS_AMOUNT,
            gasPrice: GAS_PRICE,
            nonce: index + n
        }, privKeyToPeerId(FUNDING_KEY), web3, cb), (err) => cb(err, nodes)),
    (nodes, cb) => each(nodes, (node, cb) => {
        sendTransaction({
            to: node.paymentChannels.contract._address,
            value: toWei('0.000001', 'ether'),
            gas: STAKE_GAS_AMOUNT,
            gasPrice: GAS_PRICE,
            data: node.paymentChannels.contract.methods.stakeEther().encodeABI()
        }, node.peerInfo.id, node.web3, (err) => {
            if (err)
                throw err

            node.paymentChannels.nonce = node.paymentChannels.nonce + 1

            cb()
        })
    }, (err) => cb(err, nodes)),
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
})