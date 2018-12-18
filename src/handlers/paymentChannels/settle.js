'use strict'

const pull = require('pull-stream')

const c = require('../../constants')

const CHANNEL_ID_LENGTH = 32

module.exports = (node) => node.handle(c.PROTOCOL_SETTLE_CHANNEL, (protocol, conn) => pull(
    conn,
    pull.filter((data) =>
        data.length > 0 && data.length === CHANNEL_ID_LENGTH && node.paymentChannels.has(data)
    ),
    pull.drain((channelId) => {
        console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Asked to settle channel \'' + channelId.toString('hex') + '\'.')
        node.paymentChannels.settle(channelId)
    })
))