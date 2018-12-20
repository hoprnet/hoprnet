'use strict'

const { readFileSync } = require('fs')
const { toWei } = require('web3').utils
const { waterfall, times, series, timesSeries } = require('async')

const { pubKeyToEthereumAddress } = require('../src/utils')
const { warmUpNodes } = require('./utils')

const Web3 = require('web3')
const Web3_ETH = require('web3-eth')

const { createNode } = require('../src')

const provider = new Web3.providers.WebsocketProvider('wss://rinkeby.infura.io/ws/v3/9623e709032449deb1d09a5943f4516a')
const web3_eth = new Web3_ETH(provider)

/**
 * This account is used to fund the nodes that are generated during the
 * test case.
 */
const HARDCODED_ETH_ADDRESS = '0x54C74a473d1D' + '' + '1fe0CBa42A1543FdDBbB9e7b85AC'

// Obfuscate the preivahte key a bit to make it harder
// to extract it automatically
const HARDCODED_PRIV_KEY = '0xcf295'.concat('daf6cebd') + '405d790230bed4d41380fb80c91e448c20d6cb4aaa7082cb' + Number(221 - 2).toString()

const AMOUUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 5

const CONTRACT_ADDRESS = '0x696FBD3b9471490' + 'd7807204E25FDc'.concat('c4911f3221F')

// Add the private to the Web3 wallet
web3_eth.accounts.wallet.add(HARDCODED_PRIV_KEY)
const contract = new web3_eth.Contract(JSON.parse(readFileSync(__dirname + '/utils/HoprChannel.abi')), CONTRACT_ADDRESS)

let index

waterfall([
    (cb) => web3_eth.getTransactionCount(HARDCODED_ETH_ADDRESS, cb),
    (_index, cb) => {
        index = _index
        times(AMOUUNT_OF_NODES, (_, cb) =>
            createNode({
                contract: contract,
                provider: provider
            }, cb), cb)
    },
    (nodes, cb) => warmUpNodes(nodes, cb),
    (nodes, cb) => times(nodes.length, (n, cb) => web3_eth.sendTransaction({
        from: 0,
        to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
        value: toWei('0.01', 'ether'),
        gas: 300000,
        // gasPrice: 1000000000,
        nonce: n + index
    }), (err) => cb(err, nodes)),
    (nodes, cb) => times(AMOUUNT_OF_NODES, (n, cb) => {
        web3_eth.accounts.wallet.add('0x'.concat(nodes[n].peerInfo.id.privKey.marshal().toString('hex')))

        contract.methods.stakeEther().send({
            from: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
            value: toWei('1', 'gwei'),
            gas: 230000,
            // gasPrice: 1000000000,
            nonce: n + index
        }, (err, hash) => {
            if (err) { cb(err) }
            console.log('[\'' + nodes[n].peerInfo.id.toB58String() + '\']: Staked ether with txHash \'' + hash + '\'.')
            cb(err, nodes)
        })
    }, (err) => cb(err, nodes)),
    // Wait some time to let the txs become final
    (nodes, cb) => setTimeout(cb, 7000, null, nodes),
    (nodes, cb) => series([
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo.id)

            setTimeout(cb, 11000)
        }, cb),
        (cb) => nodes[1].paymentChannels.payout(cb)
    ], cb),
], (err) => {
    console.log(err)
})