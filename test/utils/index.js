'use strict'

const { createHash } = require('crypto')
const { createNode } = require('../../src')
const { privKeyToPeerId, pubKeyToEthereumAddress, sendTransaction, log, createDirectoryIfNotExists } = require('../../src/utils')
const { STAKE_GAS_AMOUNT, GAS_PRICE } = require('../../src/constants')
const { toWei, fromWei } = require('web3-utils')
const Ganache = require('ganache-core')
const BN = require('bn.js')
const LevelDown = require('leveldown')
const chalk = require('chalk')

const MINIMAL_FUND = new BN(toWei('0.1', 'ether'))
const MINIMAL_STAKE = new BN(toWei('0.09', 'ether'))

/**
 * Allow nodes to find each other by establishing connections
 * between adjacent nodes.
 * 
 * Connection from A -> B, B -> C, C -> D, ...
 * 
 * @async
 * @param {Hopr} nodes nodes that will have open connections afterwards
 */
module.exports.warmUpNodes = async (nodes) => {
    const promises = []

    nodes.forEach((node, n) => promises.push(new Promise((resolve, reject) => {
        node.dial(nodes[(n + 1) % nodes.length].peerInfo, (err, conn) => {
            if (err)
                reject(err)

            resolve()
        })
    })))

    return Promise.all(promises)
}

/**
 * Create HOPR nodes, establish a connection between them and fund their corresponding
 * Ethereum account with some ether. And finally stake a fraction of that ether in order
 * open payment channel inside the HOPR contract.
 * 
 * @param {number} amountOfNodes number of nodes that should be generated
 * @param {object} options
 * @param {PeerId} peerId a peerId that contains public key and private key
 * @param {number} nonce the current nonce
 * @param {function} cb the function that will be called afterwards with `(err, nodes)`
 */
module.exports.createFundedNodes = async (amountOfNodes, options, peerId, nonce) => {
    const promises = []

    for (let n = 0; n < amountOfNodes; n++) {
        promises.push(createNode(Object.assign({
            id: n
        }, options)))
    }

    const nodes = await Promise.all(promises)

    await this.warmUpNodes(nodes)

    const fundBatch = []

    nodes.forEach((node, n) => fundBatch.push(this.fundNode(node, peerId, nonce + n)))

    await Promise.all(fundBatch)

    await Promise.all(nodes.map((node) => this.stakeEther(node)))

    return nodes
}

module.exports.fundNode = async (node, fundingNode, nonce) => {
    const currentBalance = new BN(await node.paymentChannels.web3.eth.getBalance(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())))

    if (!nonce)
        nonce = await node.paymentChannels.web3.eth.getTransactionCount(pubKeyToEthereumAddress(fundingNode.pubKey.marshal()))

    if (currentBalance.lt(MINIMAL_FUND)) {
        return sendTransaction({
            from: pubKeyToEthereumAddress(fundingNode.pubKey.marshal()),
            to: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            value: MINIMAL_FUND.sub(currentBalance).toString(),
            gas: STAKE_GAS_AMOUNT,
            gasPrice: process.env.GAS_PRICE,
            nonce: nonce
        }, fundingNode, node.paymentChannels.web3)
            .then(() => {
                log(node.peerInfo.id, `Received ${chalk.magenta(fromWei(MINIMAL_FUND.sub(currentBalance), 'ether'))} ETH from ${chalk.green(pubKeyToEthereumAddress(fundingNode.pubKey.marshal()))}.`)
            })
    }
}

module.exports.stakeEther = async (node) => {
    const stakedEther = await node.paymentChannels.contract.methods.states(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()))
        .call({
            from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
        }).then((result) => new BN(result.stakedEther))

    if (stakedEther.lt(MINIMAL_STAKE)) {
        return sendTransaction({
            to: process.env.CONTRACT_ADDRESS,
            value: MINIMAL_STAKE.sub(stakedEther).toString(),
            gas: STAKE_GAS_AMOUNT,
            gasPrice: process.env.GAS_PRICE,
            nonce: node.paymentChannels.nonce
        }, node.peerInfo.id, node.paymentChannels.web3)
            .then(() => {
                node.paymentChannels.nonce = node.paymentChannels.nonce + 1
                log(node.peerInfo.id, `Funded contract ${chalk.green(process.env.CONTRACT_ADDRESS)} with ${chalk.magenta(fromWei(MINIMAL_STAKE.sub(stakedEther), 'ether'))} ETH.`)
            })
    }
}

/**
 * Starts a local ganache testnet.
 * 
 * @returns {Promise} a promise that resolves once the ganache instance has been started,
 * otherwise it rejects.
 */
module.exports.startBlockchain = () => new Promise(async (resolve, reject) => {
    createDirectoryIfNotExists('db/testnet')
    const server = Ganache.server({
        accounts: [
            {
                balance: `0x${toWei(new BN(100), 'ether').toString('hex')}`,
                secretKey: process.env.FUND_ACCOUNT_PRIVATE_KEY
            }
        ],
        gasPrice: GAS_PRICE,
        db: LevelDown(`${process.cwd()}/db/testnet`),
        ws: true
    })
    server.listen(process.env.GANACHE_PORT, process.env.GANACHE_HOSTNAME, (err) => {
        if (err)
            return reject(err)

        const addr = server.address()
        console.log(`Successfully started local Ganache instance at 'ws://${addr.family === 'IPv6' ? '[' : ''}${addr.address}${addr.family === 'IPv6' ? ']' : ''}:${addr.port}'.`)

        resolve(server)
    })
})

/**
 * Starts a given amount of bootstrap servers.
 * 
 * @param {number} amountOfNodes how much bootstrap nodes
 */
module.exports.startBootstrapServers = async (amountOfNodes) => {
    const promises = []

    for (let i = 0; i < amountOfNodes; i++) {
        promises.push(createNode({
            peerId: privKeyToPeerId(
                createHash('sha256').update(i.toString()).digest()
            ),
            'bootstrap-node': true
        }).then((node) => {
            node.on('peer:connect', (peer) => {
                console.log(`Incoming connection from ${chalk.blue(peer.id.toB58String())}.`)
            })
            return node
        }))
    }

    return Promise.all(promises)
}
