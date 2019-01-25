'use strict'

const { waterfall, forever } = require('async')
const { createNode } = require('./src')
const read = require('read')
const getopts = require('getopts')
const { NET, FUNDING_KEY } = require('./src/constants')
const Multiaddr = require('multiaddr')

const options = getopts(process.argv.slice(2), {
    alias: {
        b: "bootstrap-node"
    }
})

const bootStrap = "/dns4/hopr.validity.io/tcp/9090"

console.log('Welcome to \x1b[1m\x1b[5mHOPR\x1b[0m!\n')
if (options['bootstrap-node']) {
    console.log('... running as bootstrap node')
}

const Ganache = require('ganache-core')
const Web3 = require('web3-eth')
let provider
if (NET === 'ropsten') {
    provider = ROPSTEN_WSS_URL
} else if (NET === 'ganache') {
    provider = Ganache.provider({
        accounts: [
            {
                balance: '0xd3c21bcecceda0000000',
                secretKey: FUNDING_KEY
            }
        ]
    })
}
let node, connected
waterfall([
    (cb) => createNode({
        provider: provider,
        id: `temp ${options._[0]}`
    }, cb),
    (_node, cb) => {
        node = _node
        node.once('peer:connect', (peer) => {
            console.log(`Incoming connection from ${peer.id.toB58String()}. Press enter to continue.`)
            connected = true
        })
        console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)
        connectToBootstrapNode(cb)
    },
    (cb) => {
        sendMessages(node, cb)
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
            edit: true
        }, (err, result, isDefault) => {
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
    }, (addr) => cb(null))
}

function sendMessages(node, cb) {
    forever((cb) => waterfall([
        (cb) => selectRecipient(node, cb),
        (destination, cb) => {
            console.log('Type in your message')
            read({
                edit: true
            }, (err, result, isDefault) => {
                if (err)
                    process.exit(0)
                //node.sendMessage()

                console.log(`Sending "${result}" to \x1b[34m${destination.id.toB58String()}\x1b[0m.\n`)
                cb(null)
            })
        }
    ], cb))
}

// waterfall([
//     (cb) => createNode((err, node) => {
//         if (err) { cb(err) }

//         // TODO
//         if (process.argv.length == 3) {
//             node.dial(process.argv[2], (err, conn) => cb(err, node))
//         } else {
//             console.log(renderString(node))
//             cb(err, node)
//         }
//     }, console.log)
// ], (err, node) => {
//     if (err) { throw err }

//     process.stdin.on('data', function (chunk) {
//         chunk = chunk.toString()

//         const chunks = chunk.split(' ')
//         console.log('Sending \"' + chunks[0] + '\" to ' + chunks[1])

//         node.sendMessage(chunks[0], PeerId.createFromB58String(chunks[1].trim()))
//     });
// })

// function renderString(node) {
//     let str = 'Started node ' + node.peerInfo.id.toB58String() + ' on IP address/port\n'

//     node.peerInfo.multiaddrs.forEach(addr => {
//         str = str.concat('Run \'node test_cli.js ').concat(addr.toString()).concat('\' to connect.\n')
//     })

//     return str
// }