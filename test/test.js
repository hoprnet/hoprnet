'use strict'

const { waterfall, times, parallel, map, each } = require('async')

const Hopper = require('../src/index')
const c = require('../src/constants')

const getPeerInfo = require('../src/getPeerInfo')

const Ganache = require('ganache-core')
const Eth = require('web3-eth')
const { toWei, hexToBytes } = require('web3').utils



const getContract = require('../contracts')
const { pubKeyToEthereumAddress } = require('../src/utils')

const AMOUNT_OF_NODES = Math.max(3, c.MAX_HOPS + 1)

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
        peerInfos: (cb) => times(AMOUNT_OF_NODES, (n, cb) => getPeerInfo(null, cb), cb),
        compiledContract: (cb) => getContract(cb)
    }, cb),
    ({ peerInfos, compiledContract }, cb) => {
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
        Hopper.startNode(provider, console.log, contract, cb, peerInfo)
        , cb),
    (_nodes, cb) => {
        nodes = _nodes
        warmUpNodes(nodes, cb)
    },
    (cb) => each(nodes, (node, cb) => node.paymentChannels.contract.methods.stakeMoney()
        .send({
            from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            value: toWei('1', 'ether')
        }, cb)
        .on('receipt', (receipt) => {
            console.log(receipt)
            console.log(Buffer.from(hexToBytes(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()))).toString('base64'))
            console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Staking ' + toWei('1', 'ether') + ' wei.')
        })
        , cb),
    (cb) => setTimeout(() => cb(null), 500)
], (err) => {
    if (err) { throw err }


    setTimeout(() => {
        nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo)
    }, 20000)
    nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[2].peerInfo)
})



// waterfall([
//     (cb) => parallel({
//         nodes: (cb) => waterfall([
//             (cb) => times(AMOUNT_OF_NODES, (n, cb) =>
//                 MessageDelivery.createNode(cb, console.log), cb),
//             (nodes, cb) => warmUpNodes(nodes, cb),
//             (nodes, cb) => setTimeout(() => cb(null, nodes), 200)
//         ], cb),
//         compiledContract: (cb) => getContract(cb)
//     }, cb),
//     ({ nodes, compiledContract }, cb) => {

// ], (err, contract, nodes) => {

// })

