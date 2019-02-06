'use strict'

const { waterfall, forever, each } = require('async')
const { createNode } = require('./src')
const read = require('read')
const getopts = require('getopts')
const Multiaddr = require('multiaddr')
const { pubKeyToEthereumAddress } = require('./src/utils')
const { ROPSTEN_WSS_URL } = require('./src/constants')

const options = getopts(process.argv.slice(2), {
    alias: {
        b: "bootstrap-node"
    }
})

console.log('Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n')

if (options['bootstrap-node']) {
    console.log(`... running as bootstrap node!.`)
}

options.provider = ROPSTEN_WSS_URL

const config = require('./config.json')
options.signallingPort = config.signallingPort

if (Array.isArray(options._) && options._.length > 0) {
    options.id = `temp ${options._[0]}`
}

options.addrs = []
options.signallingAddrs = []

config.interfaces.forEach((iface, index) => {
    // TODO: implement proper dual-stack
    if (Array.isArray(options._) && options._.length > 0) {
        iface.port = parseInt(iface.port) + (config.interfaces.length + 1) * parseInt(options._[0])
    }
    options.addrs.push(
        Multiaddr.fromNodeAddress({
            address: iface.host,
            port: iface.port,
        }, 'tcp')
    )
    options.signallingAddrs.push(
        Multiaddr.fromNodeAddress({
            address: iface.host,
            port: parseInt(iface.port) + 1,
        }, 'tcp')
    )
    
})


let node, connected
waterfall([
    (cb) => createNode(options, cb),
    (_node, cb) => {
        node = _node
        if (options['bootstrap-node']) {
            node.on('peer:connect', (peer) => {
                console.log(`Incoming connection from ${peer.id.toB58String()}.`)
            })
        }

        console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)
        if (!options['bootstrap-node']) {
            console.log(`Own Ethereum address:\n ${pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())}`)
            console.log('Connecting to Bootstrap node(s)...')
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
    if (!config.bootstrapServers || !Array.isArray(config.bootstrapServers))
        return cb(Error(`Unable to connect to bootstrap server. Please specify one in 'config.json'`))
    
    each(config.bootstrapServers, (addr, cb) => {
        try {
            node.dial(Multiaddr(addr), cb)
        } catch(err) {
            console.log(err)
        }
    }, cb)
    // forever((cb) => {
    //     console.log(`Please type in the Multiaddr of the node you want to connect to.`)
    //     read({
    //         edit: true,
    //         default: DEFAULT_BOOTSTRAP_ADDRESS
    //     }, (err, result) => {
    //         if (err)
    //             process.exit(0)

    //         if (!connected) {
    //             try {
    //                 const addr = new Multiaddr(result)
    //                 node.dial(addr, (err) => {
    //                     if (err) {
    //                         console.log(`\nUnable to connect to ${addr}. Please try again!`)
    //                         cb()
    //                     } else {
    //                         console.log(`\nSuccessfully connected to ${addr}.`)
    //                         cb(addr)
    //                     }
    //                 })
    //             } catch (err) {
    //                 console.log(err.message)
    //                 cb()
    //             }
    //         } else {
    //             cb(true)
    //         }
    //     })
    // }, () => cb())
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