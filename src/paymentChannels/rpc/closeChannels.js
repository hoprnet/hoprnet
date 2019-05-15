'use strict'

const pull = require('pull-stream')
const toPull = require('stream-to-pull-stream')
const lp = require('pull-length-prefixed')
const paramap = require('pull-paramap')
const chalk = require('chalk')

const BN = require('bn.js')
const { waterfall } = require('neo-async')
const Transaction = require('../../transaction')

const { pubKeyToPeerId, pubKeyToEthereumAddress, log } = require('../../utils')
const { PROTOCOL_SETTLE_CHANNEL } = require('../../constants')

const SETTLEMENT_TIMEOUT = 40000
const CHANNEL_ID_LENGTH = 32
const PREFIX = 'payments-'
const PREFIX_LENGTH = PREFIX.length

module.exports = (self) => (cb) => {
    const funds = new BN(0)
    let openChannels = 0

    self.node.db.createKeyStream({
        gt: self.RestoreTransaction(Buffer.alloc(CHANNEL_ID_LENGTH, 0)),
        lt: self.RestoreTransaction(Buffer.alloc(CHANNEL_ID_LENGTH, 255))
    })
        .on('error', (err) => cb(err))
        .on('data', async (key) => {
            const channelId = key.slice(key.length - CHANNEL_ID_LENGTH)

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

            let tx
            try {
                tx = await self.node.db.get(self.Transaction(channelId))
                    .catch((err) => {
                        if (!err.notFound)
                            throw err

                        return self.node.db.get(self.RestoreTransaction(channelId))
                    })
            } catch (err) {
                return cb(err)
            }

            openChannels = openChannels + 1
            self.onceClosed(channelId, (receivedMoney) => {
                funds.iadd(receivedMoney)
                openChannels = openChannels - 1
                if (openChannels == 0)
                    self.emit('closed channels')
            })

            const index = new BN(await self.node.db.get(self.Index(channelId)))
            if (index.gt(new BN(tx.index))) { // index > tx.index ?
                // Ask counterparty to settle payment channel because
                // last payment went to that party which means that we
                // have only one signature of the last transaction.
                const restoreTx = Transaction.fromBuffer(await self.node.db.get(self.RestoreTransaction(channelId)))
                const peerId = pubKeyToPeerId(restoreTx.counterparty)
                waterfall([
                    (cb) => self.node.peerRouting.findPeer(peerId, cb),
                    (peerInfo, cb) => self.node.dialProtocol(peerInfo, PROTOCOL_SETTLE_CHANNEL, cb),
                    (conn, cb) => {
                        const now = Date.now()

                        // TODO: Implement proper transaction handling
                        const timeout = setTimeout(self.requestClose, SETTLEMENT_TIMEOUT, channelId)

                        self.onceClosed(channelId, (err) => {
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
        })
        .on('end', () =>
            self.once('closed channels', () => cb(null, funds))
        )
}