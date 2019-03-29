'use strict'
require('dotenv').config()
const { applyEachSeries, timesSeries, times, each, waterfall } = require('neo-async')
const { createNode } = require('../../src')
const { pubKeyToEthereumAddress, sendTransaction, privKeyToPeerId, log } = require('../../src/utils')
const { GAS_PRICE, STAKE_GAS_AMOUNT } = require('../../src/constants')
const { toWei, fromWei } = require('web3-utils')
const Web3 = require('web3')
const Multiaddr = require('multiaddr')

const DEFAULT_STAKE = toWei('0.000001', 'ether')
const DEFAULT_FUND = toWei('0.05', 'ether')

/**
 * Allow nodes to find each other by establishing connections
 * between adjacent nodes.
 * 
 * Connection from A -> B, B -> C, C -> D, ...
 * 
 * @param {Hopper} nodes nodes that will have open connections afterwards
 * @param {Function} cb callback that is called when finished
 */
module.exports.warmUpNodes = (nodes, cb) =>
    times(
        nodes.length,
        (n, cb) => nodes[n].dial(nodes[(n + 1) % nodes.length].peerInfo, cb),
        (err, _) => cb(err, nodes)
    )

/**
 * Create HOPR nodes, establish a connection between them and fund their corresponding
 * Ethereum account with some ether. And finally stake a fraction of that ether in order
 * open payment channel inside the HOPR contract.
 * 
 * @param {number} amountOfNodes number of nodes that should be generated
 * @param {object} options
 * @param {string} options.provider web3.js provider, e. g. `ws://localhost:8545`
 * @param {PeerId} peerId a peerId that contains public key and private key
 * @param {number} nonce the current nonce
 * @param {function} cb the function that will be called afterwards with `(err, nodes)`
 */
module.exports.createFundedNodes = (amountOfNodes, options, peerId, nonce, cb) => {
    waterfall([
        (cb) => times(amountOfNodes, (n, cb) => waterfall([
            (cb) => {
                if (!process.env.DEMO_ACCOUNTS || process.env.DEMO_ACCOUNTS <= n)
                    return cb()

                return privKeyToPeerId(process.env[`DEMO_ACCOUNT_${n}_PRIVATE_KEY`], (err, peerId) => {
                    if (err)
                        return cb(err)

                    return cb(null, peerId)
                })
            },
            (peerId, cb) => {
                if (typeof peerId === 'function') {
                    cb = peerId
                    peerId = null
                }

                const opts = {}

                if (peerId)
                    opts.peerId = peerId

                Object.assign(opts, options, {
                    id: `temp ${n}`,
                    addrs: [
                        Multiaddr.fromNodeAddress({
                            address: "0.0.0.0",
                            port: parseInt("9091") + 2 * n
                        }, 'tcp')
                    ],
                    signallingAddrs: [
                        Multiaddr.fromNodeAddress({
                            address: "0.0.0.0",
                            port: parseInt("9091") + 2 * n + 1
                        }, 'tcp')
                    ],
                    bootstrapServers: [],
                    WebRTC: {
                        signallingServers: 3
                    }
                })

                return createNode(opts, cb)
            }
        ], cb), cb),
        (nodes, cb) => applyEachSeries([
            (cb) => this.warmUpNodes(nodes, cb),
            (cb) => timesSeries(amountOfNodes, (n, cb) =>
                sendTransaction({
                    from: pubKeyToEthereumAddress(peerId.pubKey.marshal()),
                    to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
                    value: DEFAULT_FUND,
                    gas: STAKE_GAS_AMOUNT,
                    gasPrice: GAS_PRICE,
                    nonce: nonce + n
                }, peerId, new Web3(options.provider), cb)
                    .then((receipt) => {
                        log(nodes[n].peerInfo.id, `Received ${fromWei(DEFAULT_FUND)} ETH from \x1b[32m${pubKeyToEthereumAddress(peerId.pubKey.marshal())}\x1b[0m.`)
                        cb()
                    })
                    .catch((err) => {
                        console.log(err)
                    }), cb),
                (cb) => each(nodes, (node, cb) =>
                    sendTransaction({
                        to: options.contractAddress,
                        value: DEFAULT_STAKE,
                        gas: STAKE_GAS_AMOUNT,
                        gasPrice: GAS_PRICE
                    }, node.peerInfo.id, new Web3(options.provider))
                        .then((receipt) => {
                            node.paymentChannels.nonce = node.paymentChannels.nonce + 1

                            log(node.peerInfo.id, `Funded contract \x1b[32m${options.contractAddress}\x1b[0m with ${fromWei(DEFAULT_STAKE)} ETH.`)

                            cb()
                        })
                        .catch(cb)
                , cb)
        ], (err) => cb(err, nodes))
    ], cb)
}