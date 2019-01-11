'use stric'

const { map, parallel, times, series, each, waterfall } = require('neo-async')

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
        nodes.length - 1,
        (n, cb) => nodes[n].dial(nodes[n + 1].peerInfo, cb),
        (err, _) => cb(err, nodes)
    )

const { existsSync, stat } = require('fs')
const { execFile } = require('child_process')

/**
 * Checks whether one of the src files is newer than one of
 * the artifacts.
 * 
 * @notice the method utilizes Truffle to compile the smart contracts.
 * Please make sure that Truffle is accessible by `npx`.
 * 
 * @param {Array} srcFiles the absolute paths of the source files
 * @param {Array} artifacts the absolute paths of the artifacts
 * @param {Function} cb the function that is called when finished
 */
module.exports.compileIfNecessary = (srcFiles, artifacts, cb) => {
    function compile() {
        console.log('Compiling smart contract ...')
        execFile('npx', ['truffle', 'compile'], (err, stdout, stderr) => {
            console.log(stdout)
            cb()
        })
    }

    if (artifacts.some((file) => !existsSync(file))) {
        compile()
    }

    parallel({
        srcTime: (cb) => map(srcFiles, stat, (err, stats) => {
            if (err)
                throw err

            cb(null, stats.reduce((acc, current) => Math.max(acc, current.mtimeMs), 0))
        }),
        artifactTime: (cb) => map(artifacts, stat, (err, stats) => {
            if (err)
                throw err

            cb(null, stats.reduce((acc, current) => Math.min(acc, current.mtimeMs), Date.now()))
        })
    }, (err, { srcTime, artifactTime }) => {
        if (err)
            throw err

        if (srcTime > artifactTime) {
            compile()
        } else {
            cb()
        }
    })
}

const { createNode } = require('../../src')
const { pubKeyToEthereumAddress, sendTransaction } = require('../../src/utils')
const { GAS_PRICE, STAKE_GAS_AMOUNT } = require('../../src/constants')
const { toWei } = require('web3').utils

/**
 * Create HOPR nodes, establish a connection between them and fund their corresponding
 * Ethereum account with some ether. And finally stake a fraction of that ether in order
 * open payment channel inside the HOPR contract.
 * 
 * @param {Number} amountOfNodes number of nodes that should be generated
 * @param {Object} contract an instance of Web3.js' contract interface
 * @param {Object} web3 an instance of Web3.js
 * @param {Number} nonce the current nonce
 * @param {Function} cb the function that gets called afterwards with (err, nodes)
 */
module.exports.createFundedNodes = (amountOfNodes, contract, web3, peerId, nonce, cb) => {
    waterfall([
        (cb) => times(amountOfNodes, (_, cb) =>
            createNode({
                contract: contract,
                web3: web3
            }, cb), cb),
        (nodes, cb) => parallel([
            (cb) => this.warmUpNodes(nodes, cb),
            (cb) => series([
                (cb) => times(amountOfNodes, (n, cb) =>
                    sendTransaction({
                        to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
                        value: toWei('0.05', 'ether'),
                        gas: STAKE_GAS_AMOUNT,
                        gasPrice: GAS_PRICE,
                        nonce: nonce + n
                    }, peerId, web3, cb), cb),
                (cb) => each(nodes, (node, cb) => {
                    sendTransaction({
                        to: node.paymentChannels.contract._address,
                        value: toWei('0.000001', 'ether'),
                        gas: STAKE_GAS_AMOUNT,
                        gasPrice: GAS_PRICE
                    }, node.peerInfo.id, node.web3, (err) => {
                        if (err)
                            throw err
            
                        node.paymentChannels.nonce = node.paymentChannels.nonce + 1
            
                        cb()
                    })
                }, cb)
            ], cb)
        ], (err) => cb(err, nodes))
    ], cb)
}