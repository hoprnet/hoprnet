'use strict'

const forward = require('./forward')
const acknowledge = require('./acknowledge')

module.exports = (node, output) => {
    // Registers the packet handlers if the node started as a
    // relay node.
    // This disables the relay functionality for bootstrap
    // nodes.
    if (node.peerInfo.id.privKey) {
        forward(node, output)
        acknowledge(node)
    }
}