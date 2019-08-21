'use strict'

const PacketHandler = require('./packet')
const Heartbeat = require('./heartbeat')

module.exports = (node, options) => {
    PacketHandler(node, options)
    Heartbeat(node)
}