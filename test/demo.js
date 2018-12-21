'use strict'

const { readFileSync } = require('fs')
const { toWei } = require('web3').utils
const { waterfall, times, series, timesSeries } = require('async')

const { pubKeyToEthereumAddress } = require('../src/utils')
const { warmUpNodes } = require('./utils')

const { ETH_SEND_GAS_AMOUNT, GAS_PRICE, STAKE_GAS_AMOUNT, ROPSTEN_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const Web3 = require('web3')
const web3 = new Web3(new Web3.providers.WebsocketProvider(ROPSTEN_WSS_URL))

const { createNode } = require('../src')


const AMOUUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 3

// Add the private to the Web3 wallet
web3.eth.accounts.wallet.add(HARDCODED_PRIV_KEY)
const contract = new web3.eth.Contract(JSON.parse(readFileSync(__dirname + '/utils/HoprChannel.abi')), CONTRACT_ADDRESS)

let index

waterfall([
    (cb) => web3.eth.getTransactionCount(HARDCODED_ETH_ADDRESS, cb),
    (_index, cb) => {
        index = _index
        times(AMOUUNT_OF_NODES, (_, cb) =>
            createNode({
                contract: contract,
                web3: web3
            }, cb), cb)
    },
    (nodes, cb) => warmUpNodes(nodes, cb),
    (nodes, cb) => times(nodes.length, (n, cb) => web3.eth.sendTransaction({
        from: 0,
        to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
        value: toWei('0.005', 'ether'),
        gas: ETH_SEND_GAS_AMOUNT,
        gasPrice: GAS_PRICE,
        nonce: n + index
    })
        .on('error', cb)
        .on('transactionHash', (hash) => console.log('[\'' + nodes[n].peerInfo.id.toB58String() + '\']: Received ' + toWei('0.1', 'ether') + '. TxHash \'' + hash + '\'.'))
        .on('confirmation', (n, receipt) => {
            if (n == 0)
                cb(null)
        }), (err) => cb(err, nodes)),
    (nodes, cb) => {
        nodes.forEach(node => node.web3.eth.accounts.wallet.add('0x'.concat(node.peerInfo.id.privKey.marshal().toString('hex'))))

        times(AMOUUNT_OF_NODES, (n, cb) => {


            // nodes[n].paymentChannels.nonce = node.paymentChannels.nonce + 1 // this was failing for an unknown reason, not sure why there is a nodes[n] and node 

            nodes[n].web3.eth.accounts.signTransaction({
                // nonce: nodes[n].paymentChannels.nonce, // (optional) The nonce to use when signing this transaction. Default will use web3.eth.getTransactionCount().
                value: toWei('1', 'gwei'),
                gas: STAKE_GAS_AMOUNT,
                gasPrice: GAS_PRICE,
                to: CONTRACT_ADDRESS,
                data: contract.methods.stakeEther().encodeABI()
            }, '0x'.concat(nodes[n].peerInfo.id.privKey.marshal().toString('hex')), (err, tx) => {
                nodes[n].web3.eth.sendSignedTransaction(tx.rawTransaction) // this was OGly sending a JSON object - it is expecting a string. Debugging stops here??
            })
                .on('error', cb)
                .on('transactionHash', (hash) => console.log('[\'' + nodes[n].peerInfo.id.toB58String() + '\']: Staked ' + toWei('0.1', 'ether') + '. TxHash \'' + hash + '\'.'))
                .on('confirmation', (n, receipt) => {
                    if (n == 0)
                        cb(null)
                })
        }, (err) => cb(err, nodes))
    },
    (nodes, cb) => series([
        (cb) => timesSeries(AMOUNT_OF_MESSAGES, (n, cb) => {
            nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo.id)

            setTimeout(cb, 15000)
        }, cb),
        (cb) => nodes[1].paymentChannels.payout(cb)
    ], cb),
], (err) => {
    console.log(err)
})