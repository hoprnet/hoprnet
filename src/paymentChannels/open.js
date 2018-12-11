'use strict'

const pull = require('pull-stream')

const { waterfall } = require('async')
const { toWei } = require('web3').utils
const { getId, pubKeyToEthereumAddress, deepCopy, bufferToNumber } = require('../utils')
const { recover } = require('secp256k1')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../constants')
const SIGNATURE_LENGTH = 64

const Transaction = require('../transaction')

module.exports = (self) => (to, cb) => waterfall([
    (cb) => self.node.peerRouting.findPeer(to.id, cb),
    (peerInfo, cb) => self.node.dialProtocol(peerInfo, PROTOCOL_PAYMENT_CHANNEL, cb),
    (conn, cb) => {
        const restoreTx = new Transaction()

        restoreTx.value = parseInt(toWei('1', 'shannon'))
        restoreTx.index = Math.pow(2, 32) - 1

        restoreTx.channelId = getId(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.id.pubKey.marshal()))

        restoreTx.sign(self.node.peerInfo.id.privKey.marshal())

        const tx = deepCopy(restoreTx, Transaction)
        tx.index = 0
        tx.sign(self.node.peerInfo.id.privKey.marshal())

        pull(
            pull.values([restoreTx.toBuffer(), tx.toBuffer()]),
            conn,
            pull.filter((data) => 
                data.length === SIGNATURE_LENGTH + 1
            ),
            pull.collect((err, signatures) => {
                if (signatures.length !== 2)
                    throw Error('Invalid response')

                if (recover(restoreTx.hash(), signatures[0].slice(0, SIGNATURE_LENGTH), bufferToNumber(signatures[0].slice(SIGNATURE_LENGTH)))
                    .compare(to.id.pubKey.marshal()) !== 0)
                    return false

                if (recover(tx.hash(), signatures[1].slice(0, SIGNATURE_LENGTH), bufferToNumber(signatures[1].slice(SIGNATURE_LENGTH)))
                    .compare(to.id.pubKey.marshal()) !== 0)
                    return false

                restoreTx.signature.fill(signatures[0].slice(0, SIGNATURE_LENGTH))
                restoreTx.recovery.fill(signatures[0].slice(SIGNATURE_LENGTH))

                tx.signature.fill(signatures[1].slice(0, SIGNATURE_LENGTH))
                tx.recovery.fill(signatures[1].slice(SIGNATURE_LENGTH))

                self.contract.methods.createFunded(
                    pubKeyToEthereumAddress(to.id.pubKey.marshal()),
                    toWei('1', 'shannon'),
                    tx.signature.slice(0, 32),
                    tx.signature.slice(32, 64),
                    tx.recovery
                ).send({
                    from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                    gas: 250333, // arbitrary
                    gasPrice: '30000000000000'
                }, (err, result) => {
                    if (err) { throw err }

                    self.registerSettlementListener(tx.channelId)

                    self.setRestoreTransaction(restoreTx)
                    self.set(tx)

                    cb(err, tx)
                })
            })
        )
    },
], cb)