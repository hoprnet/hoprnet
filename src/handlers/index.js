'use strict'

const AcknowledgementHandler = require('./acknowledgment')
const ForwardMessageHandler = require('./forwardMessage')
const Crawling = require('./crawler')
const Heartbeat = require('./heartbeat')
const DeliverPubKey = require('./deliverPubKey')

module.exports = (node, output) => {
    AcknowledgementHandler(node)
    ForwardMessageHandler(node, output)
    Crawling(node)
    Heartbeat(node)
    // DeliverPubKey(node)
}