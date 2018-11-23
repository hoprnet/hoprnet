'use strict'

const waterfall = require('async/waterfall')
const times = require('async/times')
// const MessageDelivery = require('../src/index')
const c = require('../src/constants')

const fs = require('fs')
const Ganache = require('ganache-core')
const Web3 = require('web3')
const web3 = new Web3(Ganache.provider(JSON.parse(fs.readFileSync(__dirname + '/' + 'config.json'))))

const getContract = require('../contracts')

web3.eth.accounts.wallet
getContract(web3, (err, contract) => {
    console.log(err, contract)

    contract.deploy().send({
        from: '0x1234567890123456789012345678901234567891',
        gas: 1500000,
        gasPrice: '30000000000000'
    })
});

// const AMOUNT_OF_NODES = Math.max(3, c.MAX_HOPS + 1)

// function warmUpNodes(nodes, cb) {
//     times(
//         nodes.length - 1,
//         (n, cb) => nodes[n].dial(nodes[n + 1].peerInfo, (err, conn) => cb(err)),
//         (err) => cb(err, nodes)
//     )
// }
// waterfall([
//     (cb) => times(AMOUNT_OF_NODES, (n, cb) => {
//         MessageDelivery.createNode(cb, console.log)
//     }, cb),
//     (nodes, cb) => warmUpNodes(nodes, cb),
//     (nodes, cb) => setTimeout(() => cb(null, nodes), 200)
// ], (err, nodes) => {
//     if (err) { throw err }

//     nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo)
// })