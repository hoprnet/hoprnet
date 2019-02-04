'use strict'

const { waterfall, forever } = require('async')
const { createNode } = require('./src')
const read = require('read')
const getopts = require('getopts')
const Multiaddr = require('multiaddr')
const { pubKeyToEthereumAddress } = require('./src/utils')

const options = getopts(process.argv.slice(2), {
    alias: {
        b: "bootstrap-node"
    }
})

const DEFAULT_BOOTSTRAP_ADDRESS = "/dns4/hopr.validity.io/tcp/9090"

console.log('Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n')

const config = {}

if (options['bootstrap-node']) {
    // console.log(`... running as bootstrap node at ${DEFAULT_BOOTSTRAP_ADDRESS}.`)
    config.peerInfo = 'BOOTSTRAP_NODE'
}

config.provider = 'ws://hopr.validity.io:8545'
if (Array.isArray(options._) && options._.length > 0) {
    config.id = `temp ${options._[0]}`
}

let node, connected
waterfall([
    (cb) => createNode(config, cb),
    (_node, cb) => {
        node = _node
        node.once('peer:connect', (peer) => {
            console.log(`Incoming connection from ${peer.id.toB58String()}. Press enter to continue.`)
            connected = true
        })
        console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)
        if (!options['bootstrap-node']) {
            console.log(`Own Ethereum address:\n ${pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())}`)
            connectToBootstrapNode(cb)
        }
    },
    (cb) => crawlNetwork(node, cb),
    (cb) => {
        if (!options['bootstrap-node']) {
            sendMessages(node, cb)
        }
    }
], (err) => {
    if (err)
        throw err
})

function selectRecipient(node, cb) {
    let peers

    forever((cb) => {
        peers = node.peerBook.getAllArray()
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
    forever((cb) => {
        console.log(`Please type in the Multiaddr of the node you want to connect to.`)
        read({
            edit: true,
            default: DEFAULT_BOOTSTRAP_ADDRESS
        }, (err, result) => {
            if (err)
                process.exit(0)

            if (!connected) {
                try {
                    const addr = new Multiaddr(result)
                    node.dial(addr, (err) => {
                        if (err) {
                            console.log(`\nUnable to connect to ${addr}. Please try again!`)
                            cb()
                        } else {
                            console.log(`\nSuccessfully connected to ${addr}.`)
                            cb(addr)
                        }
                    })
                } catch (err) {
                    console.log(err.message)
                    cb()
                }
            } else {
                cb(true)
            }
        })
    }, () => cb())
}

function sendMessages(node, cb) {
    forever((cb) => waterfall([
        (cb) => selectRecipient(node, cb),
        (destination, cb) => {
            console.log('Type in your message')
            read({
                edit: true
            }, (err, result) => {
                if (err)
                    process.exit(0)
                //node.sendMessage()

                console.log(`Sending "${result}" to \x1b[34m${destination.id.toB58String()}\x1b[0m.\n`)
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