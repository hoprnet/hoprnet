'use strict'

const { readFileSync } = require('fs')
const { toWei } = require('web3').utils
const { waterfall, times, series, timesSeries } = require('async')

const { pubKeyToEthereumAddress, sendTransaction } = require('../src/utils')
const { warmUpNodes } = require('./utils')

const { ETH_SEND_GAS_AMOUNT, GAS_PRICE, STAKE_GAS_AMOUNT, ROPSTEN_WSS_URL, HARDCODED_ETH_ADDRESS, HARDCODED_PRIV_KEY, CONTRACT_ADDRESS } = require('../src/constants')

const Web3 = require('web3')
// const web3 = new Web3('http://127.0.0.1:7545')
const web3 = new Web3(ROPSTEN_WSS_URL)

const { createNode } = require('../src')
const getContract = require('../contracts')

const AMOUUNT_OF_NODES = 4
const AMOUNT_OF_MESSAGES = 3

// Add the private to the Web3 wallet
// web3.eth.accounts.wallet.add('0xb3faf133044aebecbd8871a920818783f8e3e809a425f131046492925110ebc0')
const contract = new web3.eth.Contract(JSON.parse(readFileSync(__dirname + '/utils/HoprChannel.abi')), CONTRACT_ADDRESS)

let index

const FUNDING_ACCOUNT = HARDCODED_ETH_ADDRESS
const FUNDING_KEY = HARDCODED_PRIV_KEY
// const FUNDING_ACCOUNT = '0xB3Aa2138DE698597e2e3F84f60eF415d13731b6f'
// const FUNDING_KEY = '0xb3faf133044aebecbd8871a920818783f8e3e809a425f131046492925110ebc0'

waterfall([
    (cb) => web3.eth.getTransactionCount(FUNDING_ACCOUNT, cb),
    (_index, cb) => {
        index = _index - 1
        // getContract(cb)

        cb()
    },
    // (compiledContract, cb) => {
    //     new web3.eth.Contract(JSON.parse(compiledContract.abi.toString()))
    //         .deploy({
    //             data: compiledContract.binary.toString()
    //         })
    //         .send({
    //             from: FUNDING_ACCOUNT,
    //             gas: 3000333, // 2370333
    //             gasPrice: GAS_PRICE,
    //             nonce: index
    //         })
    //         .on('err', (err) => {
    //             console.log('err: ' + err)
    //         })
    //         .on('receipt', (receipt) => {
    //             console.log('Successfully deployed contract at address \'' + receipt.contractAddress + '\'.')
    //         })
    //         .then((contract) => cb(null, contract))
    // },
    (cb) => {
        // const contract = new web3.eth.Contract(JSON.parse(readFileSync(__dirname + '/utils/HoprChannel.abi')), CONTRACT_ADDRESS)

        times(AMOUUNT_OF_NODES, (_, cb) =>
            createNode({
                contract: contract,
                web3: web3
            }, cb), cb)
    },
    (nodes, cb) => warmUpNodes(nodes, cb),
    (nodes, cb) => timesSeries(AMOUUNT_OF_NODES, (n, cb) => {
        index = index + 1
        sendTransaction({
            to: pubKeyToEthereumAddress(nodes[n].peerInfo.id.pubKey.marshal()),
            value: toWei('0.1', 'ether'),
            gas: STAKE_GAS_AMOUNT,
            gasPrice: GAS_PRICE,
            nonce: index
        }, FUNDING_KEY, web3, cb)
    }, (err) => cb(err, nodes)),
    (nodes, cb) => timesSeries(AMOUUNT_OF_NODES, (n, cb) => {
        sendTransaction({
            to: nodes[n].paymentChannels.contract._address,
            value: toWei('0.000001', 'ether'),
            gas: STAKE_GAS_AMOUNT,
            gasPrice: GAS_PRICE,
            data: nodes[n].paymentChannels.contract.methods.stakeEther().encodeABI()
        }, nodes[n].peerInfo.id, nodes[n].web3, cb)
    }, ((err) => cb(err, nodes))),
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