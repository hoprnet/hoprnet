'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const paramap = require('pull-paramap')

const { waterfall } = require('neo-async')
const { pubKeyToPeerId, pubKeyToEthereumAddress, log } = require('../../utils')
const BN = require('bn.js')

const c = require('../../constants')

const SETTLEMENT_TIMEOUT = 40000

module.exports = (self) => (cb) => pull(
    self.getChannels(),
    paramap((data, cb) => {
        const channelId = data.key.slice(17)
        const { tx, restoreTx, index } = data.value


        waterfall([
            (cb) => self.contract.methods.channels(channelId).call({
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            }, 'latest', cb),
            (channel, cb) => {
                if (parseInt(channel.state) == 0) {
                    log(self.node.peerInfo.id, `Found orphaned payment channel ${channelId.toString('hex')} inside database. Was the node shut down inappropriately?`)
                    return self.deleteChannel(channelId, cb)
                }

                self.once(`closed ${channelId.toString('base64')}`, (receivedMoney) => {
                    // Callback just when the channel is settled, i.e. the closing listener
                    // emits the 'closed <channelId>' event.

                    cb(null, receivedMoney)
                })

                if (index.compare(tx.index) === 1) { // index > tx.index ?
                    // Ask counterparty to settle payment channel because
                    // last payment went to that party which means that we
                    // have only one signature of the last transaction.
                    waterfall([
                        (cb) => pubKeyToPeerId(restoreTx.counterparty, cb),
                        (peerId, cb) => self.node.peerRouting.findPeer(peerId, cb),
                        (peerInfo, cb) => self.node.dialProtocol(peerInfo, c.PROTOCOL_SETTLE_CHANNEL, cb),
                        (conn, cb) => {
                            const now = Date.now()

                            // TODO: Implement proper transaction handling
                            const timeout = setTimeout(self.requestClose, SETTLEMENT_TIMEOUT, channelId, true)

                            self.registerSettlementListener(channelId, (err) => {
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
                        }
                    ], (err) => {
                        if(err)
                            console.log(err)
                    })
                } else {
                    return self.requestClose(channelId)
                }
            }
        ], cb)
    }),
    pull.collect((err, values) => {
        if (err)
            return cb(err)

        return cb(null, values.reduce((acc, receivedMoney) => acc.iadd(receivedMoney), new BN(0)))
    })
)