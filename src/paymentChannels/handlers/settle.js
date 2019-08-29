'use strict'

const chalk = require('chalk')
const pull = require('pull-stream')
const paramap = require('pull-paramap')
const lp = require('pull-length-prefixed')

const fs = require('fs')
const path = require('path')
const protons = require('protons')

const { SettlementRequest, SettlementResponse } = protons(fs.readFileSync(path.resolve(__dirname, '../protos/messages.proto')))

const { PROTOCOL_SETTLE_CHANNEL } = require('../../constants')
const { log } = require('../../utils')

const CHANNEL_ID_LENGTH = 32

module.exports = node =>
    node.handle(PROTOCOL_SETTLE_CHANNEL, (protocol, conn) =>
        pull(
            conn,
            lp.decode(),
            paramap(
                (buf, cb) => {
                    let request
                    try {
                        request = SettlementRequest.decode(buf)
                    } catch (err) {
                        log(node.peerInfo.id, `Received invalid settlement request, dropping message.`)
                        return cb(null, Buffer.alloc(0))
                    }

                    if (request.channelId.length != CHANNEL_ID_LENGTH) {
                        log(node.peerInfo.id, `Received invalid settlement request, dropping message.`)
                        return cb(null, Buffer.alloc(0))
                    }

                    node.paymentChannels
                        .getLastTransaction(request.channelId)
                        .catch(err => {
                            log(
                                node.peerInfo.id,
                                `Haven't found any previous transaction for channel ${chalk.yellow(request.channelId.toString('hex'))}, dropping message.`
                            )
                            cb(null, Buffer.alloc(0))
                        })
                        .then(lastTx =>
                            cb(
                                null,
                                SettlementResponse.encode({
                                    channelId: request.channelId,
                                    transaction: lastTx.sign(node.peerInfo.id).toBuffer()
                                })
                            )
                        )
                },
                null,
                false
            ),
            lp.encode(),
            conn
        )
    )
