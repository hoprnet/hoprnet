'use strict'

const forward = require('./forward')
const acknowledge = require('./acknowledge')

module.exports = (node, options) => {
    // Registers the packet handlers if the node started as a
    // relay node.
    // This disables the relay functionality for bootstrap
    // nodes.
    if (!options['bootstrap-node']) {
        forward(node, options.output)
        acknowledge(node)
    }
}