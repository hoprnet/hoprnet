'use strict'

const pull = require('pull-stream')
const toPull = require('stream-to-pull-stream')
const lp = require('pull-length-prefixed')
const paramap = require('pull-paramap')

const BN = require('bn.js')
const { waterfall } = require('neo-async')
const Transaction = require('../../transaction')

const { pubKeyToPeerId, pubKeyToEthereumAddress, log } = require('../../utils')
const { PROTOCOL_SETTLE_CHANNEL } = require('../../constants')

const SETTLEMENT_TIMEOUT = 40000
const CHANNEL_ID_LENGTH = 32
const PREFIX = 'payments-'
const PREFIX_LENGTH = PREFIX.length

module.exports = (self) => (cb) => pull(
    toPull(
        self.node.db.createReadStream({
            gt: self.RestoreTransaction(Buffer.alloc(CHANNEL_ID_LENGTH, 0)),
            lt: self.RestoreTransaction(Buffer.alloc(CHANNEL_ID_LENGTH, 255))
        })
    ),
    paramap(async (data, cb) => {
        const channelId = data.key.slice(PREFIX_LENGTH + 10, PREFIX_LENGTH + 10 + CHANNEL_ID_LENGTH)

        const channel = await self.contract.methods.channels(channelId).call({
            from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
        }, 'latest')

        // Delete the channel in case there is no entry of it in the blockchain.
        // -> the database entry is therefore useless
        if (parseInt(channel.state) == 0) {
            log(self.node.peerInfo.id, `Found orphaned payment channel ${channelId.toString('hex')} inside database. Was the node shut down inappropriately?`)

            await self.deleteChannel(channelId)
            return
        }

        self.once(`closed ${channelId.toString('base64')}`, (receivedMoney) => {
            // Callback just when the channel is settled, i.e. the closing listener
            // emits the 'closed <channelId>' event.

            cb(null, receivedMoney)
        })

        let tx
        try {
            tx = Transaction.fromBuffer(await self.node.db.get(self.Transaction(channelId)))
        } catch (err) {
            if (!err.notFound)
                console.log(err)

            return
        }

        const index = new BN(await self.node.db.get(self.Index(channelId)))
        if (index.gt(new BN(tx.index))) { // index > tx.index ?
            // Ask counterparty to settle payment channel because
            // last payment went to that party which means that we
            // have only one signature of the last transaction.
            const restoreTx = Transaction.fromBuffer(await self.node.db.get(self.RestoreTransaction(channelId)))
            waterfall([
                (cb) => pubKeyToPeerId(restoreTx.counterparty, cb),
                (peerId, cb) => self.node.peerRouting.findPeer(peerId, cb),
                (peerInfo, cb) => self.node.dialProtocol(peerInfo, PROTOCOL_SETTLE_CHANNEL, cb),
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
                if (err) {
                    console.log(err)
                    // TODO: Handle error in a more meaningful way.
                    return cb(null, new BN(0))
                }
            })
        } else {
            self.requestClose(channelId)
        }
    }),
    pull.collect((err, values) => {
        if (err)
            return cb(err)

        return cb(null, values.reduce((acc, receivedMoney) => acc.iadd(receivedMoney), new BN(0)))
    })
)