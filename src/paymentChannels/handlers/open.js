'use strict'

const Transaction = require('../../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { sign } = require('secp256k1')
const { deepCopy, pubKeyToEthereumAddress, numberToBuffer, bufferToNumber, pubKeyToPeerId, log } = require('../../utils')
const { waterfall } = require('neo-async')
const { BN } = require('web3-utils')

const { SIGNATURE_LENGTH } = require('../../transaction')

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    lp.decode(),
    pull.asyncMap((data, cb) => {
        if (data.length !== Transaction.SIZE)
            return cb()

        const restoreTx = Transaction.fromBuffer(data)

        if (bufferToNumber(restoreTx.index) !== 1)
            return cb()

        const counterparty = restoreTx.counterparty

        waterfall([
            // checking whether the information provided by libp2p network
            // stack is plausible
            (cb) => conn.getPeerInfo(cb),
            (peerInfo, cb) => {
                if (peerInfo.id.pubKey) {
                    if (peerInfo.id.pubKey.marshal().compare(counterparty) !== 0)
                        return cb(Error('Invalid public key.'))

                    return cb()
                } else {
                    pubKeyToPeerId(counterparty, (err, peerId) => {
                        if (peerInfo.id.pubKey.bytes.compare(peerId.pubKey.bytes) !== 0)
                            return cb(Error('Invalid public key.'))

                        return cb()
                    })
                }
            },
            // Check whether the counterparty has staked enough money to open
            // the payment channel
            (cb) => node.paymentChannels.contract.methods.states(pubKeyToEthereumAddress(counterparty)).call(cb),
            (state, cb) => {
                if ((new BN(state.stakedEther)).lt(new BN(restoreTx.value)))
                    return cb(Error('Too less staked money.'))

                return cb()
            },
            // Save channel information
            (cb) => node.paymentChannels.setChannel({
                restoreTx: restoreTx,
                tx: deepCopy(restoreTx, Transaction),
                index: restoreTx.index,
                currentValue: restoreTx.value,
                totalBalance: (new BN(restoreTx.value)).imuln(2).toBuffer('be', Transaction.VALUE_LENGTH)
            }, cb),
            // Register event handler
            (cb) => {
                node.paymentChannels.setSettlementListener(restoreTx.getChannelId(node.peerInfo.id))

                const sigRestore = sign(restoreTx.hash, node.peerInfo.id.privKey.marshal())
                cb(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
            }
        ], (err, signature) => {
            if (err) {
                log(node.peerInfo.id, err.message)
                return cb(null, null)
            }

            return cb(null, signature)
        })
    }),
    pull.filter(data => Buffer.isBuffer(data)),
    lp.encode(),
    conn
))