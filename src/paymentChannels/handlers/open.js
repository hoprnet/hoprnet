'use strict'

const Transaction = require('../../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { sign } = require('secp256k1')
const { deepCopy, pubKeyToEthereumAddress, numberToBuffer, bufferToNumber, pubKeyToPeerId } = require('../../utils')
const { waterfall } = require('neo-async')
const { BN } = require('web3').utils

const { SIGNATURE_LENGTH } = require('../../transaction')

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    lp.decode(),
    pull.asyncMap((data, cb) => {
        if (data.length !== Transaction.SIZE)
            cb(true)

        const restoreTx = Transaction.fromBuffer(data)

        if (bufferToNumber(restoreTx.index) !== 1)
            cb(true, null)

        const counterparty = restoreTx.counterparty

        waterfall([
            (cb) => conn.getPeerInfo(cb),
            (peerInfo, cb) => {
                if (peerInfo.id.pubKey) {
                    if (peerInfo.id.pubKey.marshal().compare(counterparty) === 0) {
                        cb(null, restoreTx)
                    } else {
                        throw Error('Something went wrong with the signature?')
                        cb(null, null)
                    }
                } else {
                    pubKeyToPeerId(counterparty, (err, peerId) => {
                        if (peerInfo.id.pubKey.bytes.compare(peerId.pubKey.bytes) === 0) {
                            cb(null, restoreTx)
                        } else {
                            throw Error('Something went wrong with the signature?')
                            cb(null, null)
                        }
                    })
                }
            },
            // (cb) => node.paymentChannels.contract.methods.states(pubKeyToEthereumAddress(counterparty)).call(cb),
            // (state, cb) => {
            //     if ((new BN(state.stakedEther)).lt(new BN(restoreTx.value))) {
            //         cb(true)
            //     } else {
            //         cb(null, restoreTx)
            //     }
            // }
        ], cb)
    }),
    (read) => (end, reply) => waterfall([
        (cb) => {
            if (end) {
                cb(end, null)
            } else {
                read(end, cb)
            }
        },
        (restoreTx, cb) => node.paymentChannels.setChannel({
            restoreTx: restoreTx,
            tx: deepCopy(restoreTx, Transaction),
            index: restoreTx.index,
            currentValue: restoreTx.value,
            totalBalance: (new BN(restoreTx.value)).imuln(2).toBuffer('be', Transaction.VALUE_LENGTH)
        }, (err) => {
            if (err)
                throw err

            node.paymentChannels.setSettlementListener(restoreTx.getChannelId(node.peerInfo.id))

            const sigRestore = sign(restoreTx.hash, node.peerInfo.id.privKey.marshal())
            cb(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
        })
    ], reply),
    lp.encode(),
    conn
))