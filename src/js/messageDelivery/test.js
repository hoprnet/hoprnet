'use strict'

const testing = require('../testing/index')
const waterfall = require('async/waterfall')
const times = require('async/times')
const MessageDelivery = require('./index')
const c = require('./constants')

const AMOUNT_OF_NODES = Math.max(3, c.MAX_HOPS + 1)

waterfall([
    (cb) => times(AMOUNT_OF_NODES, (n, cb) => {
        MessageDelivery.createNode(cb, console.log)
    }, cb),
    (nodes, cb) => testing.warmUpNodes(nodes, cb),
    (nodes, cb) => setTimeout(() => cb(null, nodes), 80)
], (err, nodes) => {
    if (err) { throw err }

    nodes[0].sendMessage('test_test_test ' + Date.now().toString(), nodes[2].peerInfo)
})