'use strict'

const { readFileSync } = require('fs')
const { toWei } = require('web3').utils
const { pubKeyToEthereumAddress } = require('../src/utils')
const { ETH_SEND_GAS_AMOUNT, GAS_PRICE, STAKE_GAS_AMOUNT, ROPSTEN_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('./constants')

const { waterfall, times, series, timesSeries } = require('async')
const Web3_ETH = require('web3-eth')

const { createNode } = require('../src')

const web3_eth = new Web3_ETH(ROPSTEN_URL)
const { warmUpNodes } = require('./utils')

const AMOUUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 5

// Add the private to the Web3 wallet
web3_eth.accounts.wallet.add(HARDCODED_PRIV_KEY)
const contract = new web3_eth.Contract(JSON.parse(readFileSync(__dirname + '/utils/HoprChannel.abi')), CONTRACT_ADDRESS)

let nodes, index

waterfall([
    (cb) => web3_eth.getTransactionCount(HARDCODED_ETH_ADDRESS, cb),
    (_index, cb) => times(AMOUUNT_OF_NODES, (_, cb) => {
        index = _index
        createNode({
            contract: contract,
            provider: ROPSTEN_URL
        }, cb)
    }, cb),
    (nodes, cb) => warmUpNodes(nodes, cb),
    (nodes, cb) => times(nodes.length, (n, cb) => web3_eth.sendTransaction({
        from: 0,
        to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
        value: toWei('0.001', 'ether'),
        gas: ETH_SEND_GAS_AMOUNT,
        gasPrice: GAS_PRICE,
        nonce: n + index
    }, cb), (err) => cb(err, nodes)),
    // Wait some time to let the txs become final
    (nodes, cb) => setTimeout(cb, 15000, null, nodes),
    (nodes, cb) => {
        index += nodes.length

        times(AMOUUNT_OF_NODES, (n, cb) => {
            web3_eth.accounts.wallet.add('0x'.concat(nodes[n].peerInfo.id.privKey.marshal().toString('hex')))

            contract.methods.stakeEther().send({
                from: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
                value: toWei('1', 'gwei'),
                gas: STAKE_GAS_AMOUNT,
                gasPrice: GAS_PRICE,
                nonce: n + index
            }, cb)
        }, (err) => cb(err, nodes))
    },
    // Wait some time to let the txs become final
    (nodes, cb) => setTimeout(cb, 10000, null, nodes),
    (nodes, cb) => series([
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo.id)

            setTimeout(cb, 5000)
        }, cb),
        (cb) => nodes[1].paymentChannels.payout(cb)
    ], cb),
], (err) => {
    console.log(err)
})