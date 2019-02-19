'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const paramap = require('pull-paramap')


const { waterfall } = require('neo-async')
const { pubKeyToPeerId, pubKeyToEthereumAddress, log } = require('../utils')
const BN = require('bn.js')

const c = require('../constants')

const SETTLEMENT_TIMEOUT = 40000

module.exports = (self) => (cb) => pull(
    self.getChannels(),
    paramap((data, cb) => {
        const channelId = data.key.slice(17)
        const { tx, restoreTx, index } = data.value

        self.contract.methods.channels(channelId).call({
            from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
        }, 'latest', (err, channel) => {
            // check whether the channel exists
            if (parseInt(channel.state) == 0) {
                log(self.node.peerInfo.id, `Found orphaned payment channel ${channelId.toString('hex')} inside database. Was the node shut down inappropriately?`)
                return self.deleteChannel(channelId, cb)
            }

            if (index.compare(tx.index) === 1) { // index > tx.index ?
                waterfall([
                    (cb) => pubKeyToPeerId(restoreTx.counterparty, cb),
                    (peerId, cb) => self.node.peerRouting.findPeer(peerId, cb),
                    (peerInfo, cb) => self.node.dialProtocol(peerInfo, c.PROTOCOL_SETTLE_CHANNEL, cb),
                ], (err, conn) => {
                    if (err)
                        throw err

                    const now = Date.now()

                    // TODO: Implement proper transaction handling
                    const timeout = setTimeout(self.requestClose, SETTLEMENT_TIMEOUT, channelId, true)

                    self.contract.once('ClosedChannel', {
                        topics: [`0x${channelId.toString('hex')}`]
                    }, (err) => {
                        if (err)
                            throw err

                        if (Date.now() - now < SETTLEMENT_TIMEOUT) {
                            // Prevent node from settling channel itself with a probably
                            // outdated transaction
                            clearTimeout(timeout)
                        }
                    })

                    pull(
                        pull.once(channelId),
                        lp.encode(),
                        conn
                    )
                })
            } else {
                self.requestClose(channelId)
            }

            self.on(`closed ${channelId.toString('base64')}`, (receivedMoney) => {
                // Callback just when the channel is settled, i.e. the closing listener
                // emits the 'closed <channelId>' event.
    
                cb(null, receivedMoney)
            })
        })
    }),
    pull.collect((err, values) => {
        if (err)
            throw err

        cb(null, values.reduce((acc, receivedMoney) => acc.iadd(receivedMoney || new BN(0)), new BN(0)))
    })
)