'use strict'

const testing = require('../testing/index')
const waterfall = require('async/waterfall')
const times = require('async/times')
const each = require('async/each')
const pull = require('pull-stream')
const Multiaddr = require('multiaddr')
const PeerInfo = require('peer-info')
const MessageDelivery = require('./index')
const toPull = require('stream-to-pull-stream')

const AMOUNT_OF_NODES = 3
waterfall([
    (cb) => times(AMOUNT_OF_NODES, (n, cb) => {
        MessageDelivery.createNode(cb, console.log)
    }, cb),
    (nodes, cb) => testing.warmUpNodes(nodes, cb)
], (err, nodes) => {
    if (err) { throw err }

    nodes.slice(1).forEach(node => nodes[0].peerBook.put(node.peerInfo))

    nodes[0].sendMessage('test test test', nodes[2].peerInfo.id)
})