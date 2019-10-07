'use strict'

require('../config')

const chalk = require('chalk')
const { fromWei } = require('web3-utils')
const rlp = require('rlp')

const Web3 = require('web3')

const { privKeyToPeerId, log, deployContract, pubKeyToEthereumAddress } = require('../src/utils')
const { createFundedNodes, startBlockchain, startBootstrapServers } = require('./utils')

const AMOUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 1

let nonce, fundingPeer, provider, server, contractAddress

const GANACHE_SEND_TIMEOUT = 1000
const ROPSTEN_SEND_TIMEOUT = 60 * 1000

;(async () => {
    // Only for testing ===========
    process.env['NETWORK'] = 'ganache'
    // ============================

    if (process.env['NETWORK'] === 'ganache') {
        server = await startBlockchain()
        let addr = server.address()
        process.env.PROVIDER = `ws://${addr.family === 'IPv6' ? '[' : ''}${addr.address}${addr.family === 'IPv6' ? ']' : ''}:${addr.port}`
    }

    const web3 = new Web3(process.env.PROVIDER)

    fundingPeer = await privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)
    nonce = await web3.eth.getTransactionCount(await pubKeyToEthereumAddress(fundingPeer.pubKey.marshal()))

    if (process.env['NETWORK'] === 'ganache') {
        contractAddress = await deployContract(nonce, web3)
        nonce = nonce + 1
    } else {
        contractAddress = process.env[`CONTRACT_ADDRESS_${process.env['NETWORK']}`]
    }

    const bootstrapServers = await startBootstrapServers(1)

    const nodes = await createFundedNodes(
        AMOUNT_OF_NODES,
        {
            provider,
            contractAddress,
            bootstrapServers: bootstrapServers.map(node => node.peerInfo)
        },
        fundingPeer,
        nonce
    )

    let timeout
    if (process.env['NETWORK'] === 'ganache') {
        timeout = GANACHE_SEND_TIMEOUT
    } else {
        timeout = ROPSTEN_SEND_TIMEOUT
    }

    for (let n = 0; n < AMOUNT_OF_MESSAGES; n++) {
        await nodes[0].sendMessage(rlp.encode(['Psst ... secret message from Validity Labs!', Date.now().toString()]), nodes[2].peerInfo.id)

        await new Promise(resolve => setTimeout(resolve, timeout))
    }

    const closeBatch = []
    nodes.forEach(node =>
        closeBatch.push(
            node.paymentChannels.closeChannels().then(receivedMoney => {
                log(
                    node.peerInfo.id,
                    `Finally ${receivedMoney.isNeg() ? 'spent' : 'received'} ${chalk.magenta(`${fromWei(receivedMoney.abs(), 'ether')} ETH`)}.`
                )
            })
        )
    )

    await Promise.all(closeBatch)

    if (process.env['NETWORK'] === 'ganache') server.close()

    await Promise.all(nodes.concat(bootstrapServers).map(node => node.down()))
})()
