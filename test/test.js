'use strict'

const waterfall = require('async/waterfall')
const times = require('async/times')
const parallel = require('async/parallel')

const MessageDelivery = require('../src/index')
const c = require('../src/constants')

const fs = require('fs')
const Ganache = require('ganache-core')
const Web3 = require('web3')

const createKeccakHash = require('keccak')
const secp256k1 = require('secp256k1')


const getContract = require('../contracts')

const AMOUNT_OF_NODES = Math.max(3, c.MAX_HOPS + 1)

function warmUpNodes(nodes, cb) {
    times(
        nodes.length - 1,
        (n, cb) => nodes[n].dial(nodes[n + 1].peerInfo, (err, conn) => cb(err)),
        (err) => cb(err, nodes)
    )
}

function toEthereumAddress(node) {
    return '0x'.concat(
        createKeccakHash('keccak256').update(
            secp256k1.publicKeyConvert(node.peerInfo.id.pubKey.marshal(), false).slice(1)
        ).digest().slice(12).toString('hex').toUpperCase()
    )
}

function getWeb3(nodes) {
    return new Web3(Ganache.provider({
        accounts: nodes.map((node) => {
            return {
                balance: '0xd3c21bcecceda0000000',
                secretKey: '0x'.concat(node.peerInfo.id.privKey.marshal().toString('hex'))
            }
        })
    }))
}

let web3

waterfall([
    (cb) => parallel({
        nodes: (cb) => waterfall([
            (cb) => times(AMOUNT_OF_NODES, (n, cb) =>
                MessageDelivery.createNode(cb, console.log), cb),
            (nodes, cb) => warmUpNodes(nodes, cb),
            (nodes, cb) => setTimeout(() => cb(null, nodes), 200)
        ], cb),
        compiledContract: (cb) => getContract(cb)
    }, cb),
    ({ nodes, compiledContract }, cb) => {
        web3 = getWeb3(nodes)

        new web3.eth.Contract(JSON.parse(compiledContract.abi.toString()))
            .deploy({
                data: compiledContract.binary.toString()
            }).send({
                from: toEthereumAddress(nodes[0]),
                gas: 2570333, // 2370333
                gasPrice: '30000000000000'
            }).then((contract) => cb(null, contract, nodes))
    }
], (err, contract, nodes) => {
    if (err) { throw err }
    nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo)
})