'use strict'

const PacketHandler = require('./packet')
const Crawling = require('./crawler')
const Heartbeat = require('./heartbeat')
const DeliverPubKey = require('./deliverPubKey')
const STUN = require('./stun')

module.exports = (node, options) => {
    PacketHandler(node, options)
    Crawling(node)
    Heartbeat(node)
    STUN(node)

    DeliverPubKey(node)
}