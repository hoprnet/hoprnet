'use strict'

const { waterfall, times, timesSeries, series, each, parallel } = require('async')

const { createNode } = require('../src/index')
const { GAS_PRICE, MAX_HOPS } = require('../src/constants')

const Ganache = require('ganache-core')

const FUNDING_ACCOUNT = '0xB3Aa2138DE698597e2e3F84f60eF415d13731b6f'
const FUNDING_KEY = '0xb3faf133044aebecbd8871a920818783f8e3e809a425f131046492925110ebc0'

const Web3 = require('web3')
const web3 = new Web3(Ganache.provider({
    accounts: [
        {
            balance: '0xd3c21bcecceda0000000',
            secretKey: FUNDING_KEY
        }
    ]
}))

const { toWei } = require('web3').utils

const { warmUpNodes, sendTransaction } = require('./utils')

const getContract = require('../contracts')
const { pubKeyToEthereumAddress } = require('../src/utils')

const AMOUNT_OF_NODES = Math.max(3, MAX_HOPS + 1)
const AMOUNT_OF_MESSAGES = 5

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

let provider, pInfos, nodes, index = 0

waterfall([
    (cb) => getContract(cb),
    (compiledContract, cb) => sendTransaction({
        to: 0,
        gas: 3000333, // 2370333
        gasPrice: GAS_PRICE,
        nonce: index,
        data: '0x'.concat(compiledContract.binary.toString())
    }, privKeyToPeerId(FUNDING_KEY), web3, cb),
    (receipt, cb) => {
        const contract = new web3.eth.Contract(JSON.parse(readFileSync(path.resolve(__dirname, '../contracts/HoprChannel.abi'))), toChecksumAddress(receipt.contractAddress))

        times(AMOUUNT_OF_NODES, (_, cb) =>
            createNode({
                contract: contract,
                web3: web3
            }, cb), cb)
    },
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
            nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[3].peerInfo.id)

            // setTimeout(cb, 30000)
        }, cb),
        /**
         * Close the payment channel
         */
        (cb) => nodes[1].paymentChannels.payout(cb)
    ], cb)

], (err, result) => {
    if (err) { throw err }

    console.log('Received ' + result[1] + ' wei.')
})