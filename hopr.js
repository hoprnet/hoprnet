'use strict'

const dotenv = require('dotenv')
const dotenvExpand = require('dotenv-expand')

var myEnv = dotenv.config()
dotenvExpand(myEnv)

const chalk = require('chalk')
const { waterfall, forever, each } = require('neo-async')
const { createNode } = require('./src')
const read = require('read')
const getopts = require('getopts')
const Multiaddr = require('multiaddr')
const { pubKeyToEthereumAddress, randomSubset, privKeyToPeerId, sendTransaction } = require('./src/utils')
const { STAKE_GAS_AMOUNT } = require('./src/constants')
const PeerId = require('peer-id')
const BN = require('bn.js')
const { toWei, fromWei } = require('web3-utils')
const rlp = require('rlp')

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
    options.id = `temp ${options._[0]}`
}

let node
waterfall([
    (cb) => {
        if (options.id) {
            if (process.env.DEMO_ACCOUNTS && process.env.DEMO_ACCOUNTS > ~~(options._[0])) {
                privKeyToPeerId(process.env[`DEMO_ACCOUNT_${~~options._[0]}_PRIVATE_KEY`], (err, peerId) => {
                    if (err)
                        return cb(err)

                    options.peerId = peerId
                    return cb()
                })
            }
        } else {
            cb()
        }
    },
    (cb) => createNode(options, cb),
    (_node, cb) => {
        node = _node
        if (!options['bootstrap-node']) {
            node.paymentChannels.web3.eth.getBalance(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), (err, funds) => {
                if (err)
                    return cb(err)

                console.log(`Own Ethereum address:\n ${pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())}\n Private key: ${node.peerInfo.id.privKey.marshal().toString('hex')}\n Funds: ${fromWei(funds, 'ether')} ETH`)

                funds = new BN(funds)
                const minimalFunds = new BN(toWei('0.15', 'ether'))

                if (funds.lt(minimalFunds))
                    return cb(Error(`Insufficient funds. Got only ${fromWei(funds.toString(), 'ether')} ETH. Please fund the account with at least ${fromWei((minimalFunds).sub(funds), 'Ether')} ETH.`))

                return cb()
            })
        } else {
            return cb()
        }
    },
    (cb) => {
        const ownAddress = pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
        if (!options['bootstrap-node']) {
            node.paymentChannels.contract.methods
                .states(ownAddress)
                .call({
                    from: ownAddress
                }, (err, state) => {
                    if (err)
                        return cb(err)

                    console.log(` Stake: ${fromWei(state.stakedEther, 'ether')} ETH`)
                    const stakedEther = new BN(state.stakedEther)
                    if (stakedEther.lt(new BN(toWei('0.1', 'ether')))) {
                        sendTransaction({
                            from: ownAddress,
                            to: process.env.CONTRACT_ADDRESS,
                            value: toWei('0.11', 'ether'),
                            gas: STAKE_GAS_AMOUNT
                        }, node.peerInfo.id, node.paymentChannels.web3, (err, receipt) => {
                            if (err)
                                return cb(err)

                            node.paymentChannels.nonce = node.paymentChannels.nonce + 1

                            return cb()
                        })
                    } else {
                        return cb()
                    }
                })
        } else {
            return cb()
        }
    },
    (cb) => {
        if (options['bootstrap-node']) {
            node.on('peer:connect', (peer) => {
                console.log(`Incoming connection from ${peer.id.toB58String()}.`)
            })
        }

        console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)
        if (!options['bootstrap-node']) {
            console.log('Connecting to Bootstrap node(s)...')
            connectToBootstrapNode(cb)
        }
    },
    // (cb) => node.crawlNetwork(cb),
    // (cb) => node.sendMessage('123', node.peerBook.getAllArray()[0].id, cb),
    (cb) => crawlNetwork(node, cb),
    (cb) => {
        if (options['send-messages']) {
            const sendMessage = () => {
                const recipient = randomSubset(node.peerBook.getAllArray(), 1, (peerInfo) =>
                    !options.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))
                )
                return node.sendMessage('Psst ... secret message from Validity Labs!@' + Date.now().toString(), recipient[0].id, () => { })
            }
            setInterval(sendMessage, 90 * 1000)
        } else if (options['bootstrap-node']) {
            return cb()
        } else {
            return sendMessages(node, cb)
        }
    },
    (cb) => {
        if (options['bootstrap-node'])
            return cb()


        sendMessages(node, cb)
    }
], (err) => {
    if (err)
        console.log(err.message)
})

function selectRecipient(node, cb) {
    let peers

    forever((cb) => {
        peers = node.peerBook.getAllArray().filter((peerInfo) => !options.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))
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

function connectToBootstrapNode(cb) {
    if (!config.bootstrapServers || !Array.isArray(config.bootstrapServers))
        return cb(Error(`Unable to connect to bootstrap server. Please specify one in 'config.json'`))

    each(config.bootstrapServers, (addr, cb) => {
        try {
            node.dial(Multiaddr(addr), cb)
        } catch (err) {
            console.log(err)
        }
    }, cb)
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

                console.log(`Sending "${message}" to \x1b[34m${destination.id.toB58String()}\x1b[0m.\n`)

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