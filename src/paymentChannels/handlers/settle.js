'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

const { PROTOCOL_SETTLE_CHANNEL } = require('../../constants')
const { log } = require('../../utils')

const CHANNEL_ID_LENGTH = 32

module.exports = (node) => node.handle(PROTOCOL_SETTLE_CHANNEL, (protocol, conn) => pull(
    conn,
    lp.decode({
        maxLength: CHANNEL_ID_LENGTH
    }),
    pull.drain((channelId) => {
        log(node.peerInfo.id, `Asked to settle channel \x1b[33m${channelId.toString('hex')}\x1b[0m.`)
        node.paymentChannels.requestClose(channelId)
    })
))