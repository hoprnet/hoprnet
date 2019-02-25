'use strict'

const PacketHandler = require('./packet')
const Crawling = require('./crawler')
const Heartbeat = require('./heartbeat')
const DeliverPubKey = require('./deliverPubKey')
const PublicIp = require('./publicIp')

module.exports = (node, options) => {
    PacketHandler(node, options)
    Crawling(node)
    Heartbeat(node)
    PublicIp(node)

    DeliverPubKey(node)
}