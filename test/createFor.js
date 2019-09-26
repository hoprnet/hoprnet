require('../config')

const chalk = require('chalk')
const { fromWei, toWei } = require('web3-utils')
const rlp = require('rlp')

const BN = require('bn.js')
const Web3 = require('web3')

const { createNode } = require('../src')

const { pubKeyToEthereumAddress, privKeyToPeerId, log, deployContract } = require('../src/utils')
const { startBootstrapServers, startBlockchain } = require('./utils')

const AMOUNT_OF_NODES = 4

;(async () => {
    const bootstrapServers = await startBootstrapServers(1)

    const web3 = new Web3(process.env.PROVIDER)

    await startBlockchain()

    const fundingNode = await privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY)

    let nonce = await web3.eth.getTransactionCount(pubKeyToEthereumAddress(fundingNode.pubKey.marshal()))

    const contractAddress = await deployContract(nonce++, web3)

    const abi = require('../build/contracts/HoprChannel.json').abi
    const contract = new web3.eth.Contract(abi, contractAddress, {
        from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal())
    })

    const promises = []
    for (let i = 0; i < AMOUNT_OF_NODES; i++) {
        promises.push(
            createNode({
                id: i,
                contractAddress,
                bootstrapServers: bootstrapServers.map(node => node.peerInfo)
            })
        )
    }

    const nodes = await Promise.all(promises)

    const batch = new web3.eth.BatchRequest()

    let partyA, partyB
    for (let i = 0; i < AMOUNT_OF_NODES; i++) {
        partyA = pubKeyToEthereumAddress(nodes[i].peerInfo.id.pubKey.marshal())
        for (let j = i + 1; j < AMOUNT_OF_NODES; j++) {
            partyB = pubKeyToEthereumAddress(nodes[j].peerInfo.id.pubKey.marshal())

            batch.add(openChannelFor(fundingNode, contract, nonce++, partyA, partyB))
        }
    }

    await batch.execute()
})()

function openChannelFor(fundingNode, contract, nonce, partyA, partyB) {
    return contract.methods.createFor(partyA, partyB).send.request({
        from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal()),
        gas: 190000,
        //gasPrice: process.env.GAS_PRICE,
        value: toWei('0.2', 'ether'),
        nonce: `0x${new BN(nonce).toBuffer('be').toString('hex')}`
    })
}