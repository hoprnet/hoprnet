'use strict'

const { waterfall, times, timesSeries, series, each, parallel, map } = require('async')

const Hopper = require('../src/index')
const c = require('../src/constants')

const getPeerInfo = require('../src/getPeerInfo')

const Ganache = require('ganache-core')
const Eth = require('web3-eth')
const { toWei, hexToBytes } = require('web3').utils

const getContract = require('../contracts')
const { pubKeyToEthereumAddress } = require('../src/utils')

const AMOUNT_OF_NODES = Math.max(3, c.MAX_HOPS + 1)
const AMOUNT_OF_MESSAGES = 4

/**
 * Allow nodes to find each other by establishing connections
 * between adjacent nodes.
 * 
 * Connection from A -> B, B -> C, C -> D, ...
 * 
 * @param {Hopper} nodes nodes that will have open connections afterwards
 * @param {Function} cb callback that is called when finished
 */
function warmUpNodes(nodes, cb) {
    times(
        nodes.length - 1,
        (n, cb) => nodes[n].dial(nodes[n + 1].peerInfo, cb),
        (err, _) => cb(err)
    )
}

function getGUIGanacheProvider() {
    return 'http://localhost:7545'
}

/**
 * Feed Ganache-Core with accounts and balances. The
 * accounts are derived from the peerInfo
 * 
 * @param {PeerInfo} peerInfos PeerInfo instances that are used to derive accounts
 */
function getWeb3Provider(peerInfos) {
    return Ganache.provider({
        accounts: peerInfos.map((peerInfo) => {
            return {
                balance: '0xd3c21bcecceda0000000',
                secretKey: '0x'.concat(peerInfo.id.privKey.marshal().toString('hex'))
            }
        })
    })
}

let provider, pInfos, nodes

waterfall([
    (cb) => parallel({
        /**
         * Compile contract, generate peer config
         */
        peerInfos: (cb) => times(AMOUNT_OF_NODES, (n, cb) => getPeerInfo(null, cb), cb),
        compiledContract: (cb) => getContract(cb)
    }, cb),
    ({ peerInfos, compiledContract }, cb) => {
        /**
         * Register Web3 provider and deploy contract.
         */
        pInfos = peerInfos

        // provider = getGUIGanacheProvider()
        provider = getWeb3Provider(pInfos)
        const eth = new Eth(provider)

        new eth.Contract(JSON.parse(compiledContract.abi.toString()))
            .deploy({
                data: compiledContract.binary.toString()
            })
            .send({
                from: pubKeyToEthereumAddress(pInfos[0].id.pubKey.marshal()),
                gas: 3000333, // 2370333
                gasPrice: '30000000000000'
            })
            .on('receipt', (receipt) => {
                console.log('Successfully deployed contract at address \'' + receipt.contractAddress + '\'.')
            })
            .then((contract) => cb(null, contract))
    },
    (contract, cb) => map(pInfos, (peerInfo, cb) =>
        /**
         * Start nodes, node will start listening on the network interface
         */
        Hopper.startNode(provider, console.log, contract, cb, peerInfo)
        , cb),
    (_nodes, cb) => parallel([
        (cb) => {
            /**
             * Bootstrapping
             */
            nodes = _nodes
            warmUpNodes(nodes, cb)
        },
        (cb) => each(nodes, (node, cb) => node.paymentChannels.contract.methods.stakeEther()
            /**
             * Stake ether for each node
             */
            .send({
                from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                value: toWei('1', 'ether')
            }, cb)
            .on('receipt', (receipt) => {
                console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Own Ethereum address is \'' + pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()) + '\'.')
                console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Staking ' + toWei('1', 'ether') + ' wei.')
            }), cb),
        /**
         * Wait some time until the connections are open
         */
        (cb) => setTimeout(() => cb(null), 500)
    ], cb),
    (_, cb) => series([
        /**
         * Send dummy messages every other second
         */
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo)

            setTimeout(cb, 2000)
        }, cb),
        /**
         * Close the payment channel
         */
        (cb) => nodes[1].paymentChannels.payout(cb)
    ], cb)

], (err) => {
    if (err) { throw err }
})