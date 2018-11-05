'use strict'

const pull = require('pull-stream')

const constants = require('../constants')

module.exports = (node) => node.handle(constants.PROTOCOL_ACKNOWLEDGEMENT, (protocol, conn) => {
    pull(
        conn,
        pull.drain(data => {
            console.log('Acknowledgement ' + data)
        })
    )
})