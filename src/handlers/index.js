'use strict'

const PacketHandler = require('./packet')
const Heartbeat = require('./heartbeat')
const DeliverPubKey = require('./deliverPubKey')
const PublicIp = require('./publicIp')

module.exports = (node, options) => {
    PacketHandler(node, options)
    Heartbeat(node)
    PublicIp(node)

    DeliverPubKey(node)
}