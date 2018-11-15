'use strict'

const waterfall = require('async/waterfall')
const Multiaddr = require('multiaddr')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const MessageDelivery = require('../src/index')

process.stdin.setEncoding('utf8');

waterfall([
    (cb) => MessageDelivery.createNode((err, node) => {
        if (err) { cb(err) }

        // TODO
        if (process.argv.length == 3) {
            node.dial(process.argv[2], (err, conn) => cb(err, node))
        } else {
            console.log(renderString(node))
            cb(err, node)
        }
    }, console.log)
], (err, node) => {
    if (err) { throw err }

    process.stdin.on('data', function (chunk) {
        chunk = chunk.toString()

        const chunks = chunk.split(' ')
        console.log('Sending \"' + chunks[0] + '\" to ' + chunks[1])

        node.sendMessage(chunks[0], PeerId.createFromB58String(chunks[1].trim()))
    });
})

function renderString(node) {
    let str = 'Started node ' + node.peerInfo.id.toB58String() + ' on IP address/port\n'

    node.peerInfo.multiaddrs.forEach(addr => {
        str = str.concat('Run \'node test_cli.js ').concat(addr.toString()).concat('\' to connect.\n')
    })

    return str
}