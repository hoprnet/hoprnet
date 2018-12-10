'use strict'

const pull = require('pull-stream')

const { waterfall } = require('async')
const { toWei } = require('web3').utils
const { getId, pubKeyToEthereumAddress } = require('../utils')
const { verify } = require('secp256k1')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')
const SIGNATURE_LENGTH = 64

const Transaction = require('../transaction')

module.exports = (self) => (to, cb) => waterfall([
    (cb) => self.node.peerRouting.findPeer(to.id, cb),
    (peerInfo, cb) => self.node.dialProtocol(peerInfo, PROTOCOL_PAYMENT_CHANNEL, cb),
    (conn, cb) => {
        const tx = new Transaction()

        tx.value = parseInt(toWei('1', 'shannon'))
        tx.index = Math.pow(2, 32) - 1

        tx.channelId = getId(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.id.pubKey.marshal()))

        tx.sign(self.node.peerInfo.id.privKey.marshal())

        pull(
            pull.once(tx.toBuffer()),
            conn,
            pull.filter((data) =>
                data.length === SIGNATURE_LENGTH &&
                verify(tx.hash(), data, to.id.pubKey.marshal())
            ),
            pull.drain((signature) => {
                tx.signature.fill(signature, 0, SIGNATURE_LENGTH)

                self.set(tx)
                self.registerSettlementListener(tx.channelId)

                self.contract.methods.create(pubKeyToEthereumAddress(to.id.pubKey.marshal()), toWei('1', 'shannon')).send({
                    from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                    gas: 250333, // arbitrary
                    gasPrice: '30000000000000'
                }, (err, receipt) => cb(err, tx, receipt))
            }))
    },
], cb)