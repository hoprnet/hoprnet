'use strict'

const pull = require('pull-stream')

const c = require('../../constants')
const { log } = require('../../utils')

const CHANNEL_ID_LENGTH = 32

module.exports = (node) => node.handle(c.PROTOCOL_SETTLE_CHANNEL, (protocol, conn) => pull(
    conn,
    pull.filter((data) =>
        data.length > 0 && data.length === CHANNEL_ID_LENGTH && node.paymentChannels.has(data)
    ),
    pull.drain((channelId) => {
        log(node.peerInfo.id, `Asked to settle channel \x1b[33m${channelId.toString('hex')}\x1b[0m.`)
        node.paymentChannels.settle(channelId)
    })
))