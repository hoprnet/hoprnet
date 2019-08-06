'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

const myEnv = dotenv.config()
dotenvExpand(myEnv)

const chalk = require('chalk')
const { fromWei } = require('web3-utils')
const rlp = require('rlp')

const Web3 = require('web3')

const { privKeyToPeerId, log, deployContract } = require('../src/utils')
const { createFundedNodes, startBlockchain, startBootstrapServers } = require('./utils')

const AMOUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 1

console.log(
    'Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n' +
    'Please wait some time until the node is set up.\n' +
    '\x1b[2mThis may take some time ...\n' +
    'Meanwhile you can start reading the wiki at https://github.com/validitylabs/hopr/wiki\x1b[0m\n')

let nonce, fundingPeer, provider, server, contractAddress

const GANACHE_SEND_TIMEOUT = 1000
const ROPSTEN_SEND_TIMEOUT = 60 * 1000

const main = async () => {
    // Only for testing ===========
    process.env.NETWORK = 'ganache'
    // ============================

    if (process.env.NETWORK === 'ganache') {
        server = await startBlockchain()
        let addr = server.address()
        process.env.PROVIDER = `ws://${addr.family === 'IPv6' ? '[' : ''}${addr.address}${addr.family === 'IPv6' ? ']' : ''}:${addr.port}`
    }

    const web3 = new Web3(process.env.PROVIDER)

    fundingPeer = await privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)
    nonce = await web3.eth.getTransactionCount(process.env.FUND_ACCOUNT_ETH_ADDRESS)

    if (process.env.NETWORK === 'ganache') {
        contractAddress = await deployContract(nonce, web3)
        nonce = nonce + 1
    } else {
        contractAddress = process.env[`CONTRACT_ADDRESS_${process.env.NETWORK}`]
    }

    const bootstrapServers = await startBootstrapServers(1)

    const nodes = await createFundedNodes(AMOUNT_OF_NODES, {
        provider: provider,
        contractAddress: contractAddress,
        bootstrapServers: bootstrapServers.map((node) => node.peerInfo)
    }, fundingPeer, nonce)

    let timeout
    if (process.env.NETWORK === 'ganache') {
        timeout = GANACHE_SEND_TIMEOUT
    } else {
        timeout = ROPSTEN_SEND_TIMEOUT
    }

    for (let n = 0; n < AMOUNT_OF_MESSAGES; n++) {
        nodes[0].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[2].peerInfo.id)

        await new Promise((resolve) => setTimeout(resolve, timeout))
    }

    await new Promise((resolve) => setTimeout(resolve, timeout))

    const closeBatch = []
    nodes.forEach((node) => {
        closeBatch.push(node.paymentChannels.closeChannels()
            .then((receivedMoney) => {
                log(node.peerInfo.id, `Finally ${receivedMoney.isNeg() ? 'spent' : 'received'} ${chalk.magenta(`${fromWei(receivedMoney.abs(), 'ether')} ETH`)}.`)
            }))
    })

    await Promise.all(closeBatch)

    nodes.forEach((node) => node.stop())

    if (process.env.NETWORK === 'ganache')
        setTimeout(server.close, 2000)
}

main()