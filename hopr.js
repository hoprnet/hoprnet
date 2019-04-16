'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

var myEnv = dotenv.config()
dotenvExpand(myEnv)

const chalk = require('chalk')
const { waterfall, forever, each } = require('neo-async')
const Hopr = require('./src')
const read = require('read')
const getopts = require('getopts')
const { pubKeyToEthereumAddress, randomSubset, privKeyToPeerId, sendTransaction } = require('./src/utils')
const { STAKE_GAS_AMOUNT } = require('./src/constants')
const PeerId = require('peer-id')
const BN = require('bn.js')
const { toWei, fromWei } = require('web3-utils')
const rlp = require('rlp')

const MINIMAL_FUNDS = new BN(toWei('0.15', 'ether'))

const options = getopts(process.argv.slice(2), {
    alias: {
        b: "bootstrap-node",
        m: "send-messages",
    }
})

console.log(`Welcome to ${chalk.bold('HOPR')}!\n`)

if (options['bootstrap-node'])
    console.log(`... running as bootstrap node!.`)

if (Array.isArray(options._) && options._.length > 0) {
    options.id = Number.parseInt(options._[0])
}

function connectToBootstrapNode(node) {
    return Promise.all([
        node.bootstrapServers.forEach((addr) => new Promise((resolve, reject) => {
            node.dial(addr, (err, conn) => {
                if (err)
                    return reject(conn)

                resolve()
            })
        }))
    ])
}

async function main() {
    let node = await new Promise((resolve, reject) => {
        Hopr.createNode(options, (err, node) => {
            if (err)
                return reject(err)

            resolve(node)
        })
    })
    
    const ownAddress = pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())

    console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)

    if (options['bootstrap-node']) {
        node.on('peer:connect', (peer) => {
            console.log(`Incoming connection from ${peer.id.toB58String()}.`)
        })
    }

    if (!options['bootstrap-node']) {
        let funds
        try {
            funds = await node.paymentChannels.web3.eth.getBalance(ownAddress)
        } catch (err) {
            console.log(err)
            return
        }

        console.log(
            `Own Ethereum address:\n` +
            `\t${pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())}\n` +
            `\tPrivate key: ${node.peerInfo.id.privKey.marshal().toString('hex')}\n` +
            `\tFunds: ${fromWei(funds, 'ether')} ETH`
        )

        funds = new BN(funds)

        if (funds.lt(MINIMAL_FUNDS))
            throw Error(`Insufficient funds. Got only ${fromWei(funds.toString(), 'ether')} ETH. Please fund the account with at least ${fromWei((MINIMAL_FUNDS).sub(funds), 'Ether')} ETH.`)

        let state
        try {
            state = await node.paymentChannels.contract.methods.states(ownAddress).call({ from: ownAddress })
        } catch (err) {
            console.log(err)
            return
        }

        const stakedEther = new BN(state.stakedEther)

        if (stakedEther.lt(new BN(toWei('0.1', 'ether')))) {
            let receipt
            try {
                receipt = await sendTransaction({
                    from: ownAddress,
                    to: process.env.CONTRACT_ADDRESS,
                    value: toWei('0.11', 'ether'),
                    gas: STAKE_GAS_AMOUNT
                }, node.peerInfo.id, node.paymentChannels.web3)
            } catch (err) {
                console.log(err)
                return
            }

            node.paymentChannels.nonce = node.paymentChannels.nonce + 1
        }

        console.log(`\tStake: ${fromWei(state.stakedEther, 'ether')} ETH`)

        console.log('Connecting to Bootstrap node(s)...')
        try {
            await connectToBootstrapNode(node)
        } catch (err) {
            console.log(err)
            return
        }

        await new Promise((resolve, reject) => crawlNetwork(node, (err) => {
            if (err)
                return reject(err)

            return resolve()
        }))

        if (options['send-messages']) {
            const sendMessage = () => {
                const recipient = randomSubset(node.peerBook.getAllArray(), 1, (peerInfo) =>
                    !options.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))
                )
                return node.sendMessage('Psst ... secret message from Validity Labs!@' + Date.now().toString(), recipient[0].id, () => { })
            }
            setInterval(sendMessage, 90 * 1000)
        } else if (options['bootstrap-node']) {
            return
        } else {
            return sendMessages(node, () => {})
        }
    }
}

main()


function selectRecipient(node, cb) {
    let peers

    forever((cb) => {
        peers = node.peerBook.getAllArray().filter((peerInfo) => !node.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))
        )
        console.log(
            peers.reduce((acc, peerInfo, index) => {
                return acc.concat(`[${index + 1}] \x1b[34m${peerInfo.id.toB58String()}\x1b[0m\n`)
            }, '')
        )

        read({
            edit: true
            // default: '\x1b[5mHOPR\x1b[0m!\n'
        }, (err, result, isDefault) => {
            if (err)
                process.exit(0)

            const choice = parseInt(result, 10)

            if (!Number.isInteger(choice) || choice <= 0 || choice > peers.length) {
                console.log('Invalid choice. Try again!')
                cb()
            } else {
                console.log(`Sending to \x1b[34m${peers[choice - 1].id.toB58String()}\x1b[0m.`)
                cb(peers[choice - 1])
            }
        })
    }, (peerId) => {
        cb(null, peerId)
    })
}


function sendMessages(node, cb) {
    forever((cb) => waterfall([
        (cb) => selectRecipient(node, cb),
        (destination, cb) => {
            console.log('Type in your message')
            read({
                edit: true
            }, (err, message) => {
                if (err)
                    process.exit(0)

                console.log(`Sending "${message}" to \x1b[34m${destination.id.toB58String()} \x1b[0m.\n`)

                const encodedMessage = rlp.encode([message, Date.now().toString()])
                node.sendMessage(encodedMessage, destination.id, cb)

                cb()
            })
        }
    ], cb))
}

function crawlNetwork(node, cb) {
    forever((cb) => {
        console.log('Crawl network. Enter Y to crawl network, and N to proceed')
        read({}, (err, result) => {
            if (err)
                process.exit(0)

            if (result.toLowerCase() === 'y') {
                node.crawlNetwork((err) => {
                    if (err)
                        console.log(err.message)

                    return cb()
                })
            } else {
                return cb(true)
            }
        })
    }, (err) => cb())
}