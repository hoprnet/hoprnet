'use strict'

const AcknowledgementHandler = require('./acknowledgment')
const ForwardMessageHandler = require('./forwardMessage')

module.exports = (node, output, callback) => {
    AcknowledgementHandler(node)
    ForwardMessageHandler(node, output)

    callback(null, node)
}