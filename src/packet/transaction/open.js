'use strict'

const pull = require('pull-stream')

const { waterfall } = require('async')
const { toWei } = require('web3').utils
const { getId, pubKeyToEthereumAddress } = require('../../utils')

const secp256k1 = require('secp256k1')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const SIGNATURE_LENGTH = 64

module.exports = (Transaction, to, node, cb) => waterfall([
    (cb) => node.peerRouting.findPeer(to, cb),
    (peerInfo, cb) => node.dial(peerInfo, PROTOCOL_PAYMENT_CHANNEL, cb),
    (conn, cb) => {
        const tx = new Transaction()

        tx.value = parseInt(toWei('1', 'shannon'))
        tx.index = Math.pow(2, 32) - 1

        tx.channelId = getId(
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal()))

        tx.sign(node, to)



        pull(
            pull.once(tx.toBuffer()),
            conn,
            pull.filter((data) => data.length === SIGNATURE_LENGTH),
            pull.filter((signature) => secp256k1.verify(tx.hash(node, to), signature, to.pubKey.marshal())),
            pull.drain(signature => waterfall([
                (cb) => node.contract.methods.create(pubKeyToEthereumAddress(to.pubKey.marshal()), toWei('1', 'shannon')).send({
                    from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()),
                    gas: 2570333, // 2370333
                    gasPrice: '30000000000000'
                }, cb),
                (receipt, cb) => {
                    console.log(receipt)
                    console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Funding payment channel with node \'' + to.toB58String() + '\' with ' + toWei('1', 'shannon') + ' wei.')
                    tx.signature.fill(signature, 0, SIGNATURE_LENGTH)

                    node.openPaymentChannels.set(tx.channelId.toString('base64'), tx)

                    cb(null, tx)
                }
            ], cb))
        )
    },
], cb)


// irgendwo kommt der pubKey her??
// on-chain actions!!!