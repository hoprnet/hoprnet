'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

var myEnv = dotenv.config()
dotenvExpand(myEnv)

const { waterfall, timesSeries, times, map, parallel } = require('neo-async')
const rlp = require('rlp')

const Web3 = require('web3')

const { privKeyToPeerId, log, deployContract } = require('../src/utils')
const { createFundedNodes, startTestnet } = require('./utils') 

const AMOUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 1

console.log(
    'Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n' +
    'Please wait some time until the node is set up.\n' +
    '\x1b[2mThis may take some time ...\n' +
    'Meanwhile you can start reading the wiki at https://github.com/validitylabs/messagingProtocol/wiki\x1b[0m\n')

let index, fundingPeer, provider, server, web3

waterfall([
    async (cb) => {
        if (process.env.NETWORK === 'ganache')
            server = await startTestnet()
        
        web3 = new Web3(process.env.PROVIDER)
    
        parallel({
            fundingPeer: (cb) => privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY, cb),
            index: (cb) => web3.eth.getTransactionCount(process.env.FUND_ACCOUNT_ETH_ADDRESS, cb),
        }, cb)
    },
    (results, cb) => {
        fundingPeer = results.fundingPeer
        index = results.index

        if (process.env.NETWORK === 'ganache') {
            deployContract(index, web3).then((address) => {
                index = index + 1

                return cb(null, address)
            }, cb)
        } else {
            return cb(null, process.env.CONTRACT_ADDRESS)
        }
    },
    (contractAddress, cb) => createFundedNodes(AMOUNT_OF_NODES, {
        provider: provider,
        contractAddress: contractAddress
    }, fundingPeer, index, cb),
    (nodes, cb) => waterfall([
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[2].peerInfo.id)

            if (process.env.NETWORK === 'ganache') {
                setTimeout(cb, 2000)
            } else {
                throw Error('TODO')
                setTimeout(cb, 60 * 1000)
            }
        }, () => setTimeout(cb, 5000)),
        (cb) => map(nodes, (node, cb) => node.paymentChannels.closeChannels(cb), cb),
        (results, cb) => times(AMOUNT_OF_NODES, (n, cb) => {
            log(nodes[n].peerInfo.id, `Finally ${results[n].isNeg() ? 'spent' : 'received'} \x1b[35m\x1b[1m${results[n].abs()} wei\x1b[0m.`)
            nodes[n].stop(cb)
        }, cb)
    ], (err, _) => cb(err)),
    (cb) => {
        if (process.env.NETWORK === 'ganache') {
            setTimeout(server.close, 2000, cb)
        } else {
            return cb()
        }
    }
], (err) => {
    if (err)
        throw err
})