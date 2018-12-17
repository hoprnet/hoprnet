'use strict'

const AcknowledgementHandler = require('./acknowledgment')
const ForwardMessageHandler = require('./forwardMessage')
const Crawling = require('./crawler')
const DeliverPubKey = require('./deliverPubKey')
const PaymentChannel = require('./paymentChannels')

module.exports = (node, output, callback) => {
    AcknowledgementHandler(node)
    ForwardMessageHandler(node, output)
    Crawling(node)
    DeliverPubKey(node)
    PaymentChannel(node)

    callback(null, node)
}