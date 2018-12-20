'use strict'

const pull = require('pull-stream')

const { waterfall } = require('async')
const { toWei } = require('web3').utils
const { getId, pubKeyToEthereumAddress, deepCopy, bufferToNumber } = require('../utils')
const { recover } = require('secp256k1')

const { PROTOCOL_PAYMENT_CHANNEL, DEFAULT_GAS_AMOUNT, GAS_PRICE } = require('../constants')
const SIGNATURE_LENGTH = 64

const Transaction = require('../transaction')

module.exports = (self) => (to, cb) => waterfall([
    (cb) => self.node.peerRouting.findPeer(to.id, cb),
    (peerInfo, cb) => self.node.dialProtocol(peerInfo, PROTOCOL_PAYMENT_CHANNEL, cb),
    (conn, cb) => {
        const restoreTx = new Transaction()

        restoreTx.value = parseInt(toWei('1', 'shannon'))
        restoreTx.index = 1

        restoreTx.channelId = getId(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.id.pubKey.marshal()))

        restoreTx.sign(self.node.peerInfo.id.privKey.marshal())

        // TODO this might fail when opening more than one channel at the same time
        self.nonce = self.nonce + 1

        pull(
            pull.once(restoreTx.toBuffer()),
            conn,
            pull.filter((data) =>
                Buffer.isBuffer(data) &&
                data.length === SIGNATURE_LENGTH + 1 &&
                recover(restoreTx.hash, data.slice(0, SIGNATURE_LENGTH), bufferToNumber(data.slice(SIGNATURE_LENGTH)))
                    .compare(to.id.pubKey.marshal()) === 0
            ),
            pull.collect((err, signatures) => {
                if (signatures.length !== 1)
                    throw Error('Invalid response')

                restoreTx.signature.fill(signatures[0].slice(0, SIGNATURE_LENGTH))
                restoreTx.recovery.fill(signatures[0].slice(SIGNATURE_LENGTH))

                self.contract.methods.createFunded(
                    pubKeyToEthereumAddress(to.id.pubKey.marshal()),
                    toWei('1', 'shannon'),
                    restoreTx.signature.slice(0, 32),
                    restoreTx.signature.slice(32, 64),
                    restoreTx.recovery
                ).send({
                    from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                    gas: DEFAULT_GAS_AMOUNT, // arbitrary
                    gasPrice: GAS_PRICE,
                    nonce: self.nonce
                })
                    .on('error', cb)
                    .on('transactionHash', (hash) => console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Opened payment channel with txHash \'' + hash + '\'.'))
                    .on('confirmation', (n, receipt) => {
                        if (n == 0) {
                            self.setSettlementListener(restoreTx.channelId)
                            self.setRestoreTransaction(restoreTx)

                            const tx = deepCopy(restoreTx, Transaction)
                            self.set(tx)

                            console.log('[\'' + self.node.peerInfo.id.toB58String() + '\']: Opened payment channel with txHash \'' + hash + '\'.')

                            cb(null, tx)
                        }
                    })
            })
        )
    }
], cb)