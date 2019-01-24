'use strict'

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { pubKeyToPeerId } = require('../utils')
const { decode } = require('multihashes')
const { BN } = require('web3-utils')

const { PROTOCOL_ACKNOWLEDGEMENT } = require('../constants')
const { bufferXOR, hash, log } = require('../utils')
const Acknowledgement = require('../acknowledgement')

module.exports = (node) => node.handle(PROTOCOL_ACKNOWLEDGEMENT, (protocol, conn) => pull(
    conn,
    lp.decode(),
    pull.filter(data =>
        data.length > 0 && data.length === Acknowledgement.SIZE
    ),
    pull.map(data => Acknowledgement.fromBuffer(data)),
    pull.drain(ack => {
        if (ack.challengeSigningParty.compare(node.peerInfo.id.pubKey.marshal()) !== 0)
            throw Error('General error.')

        node.pendingTransactions.getEncryptedTransaction(ack.hashedKey, (err, record) => {
            if (!record)
                throw Error('General error.')

            const { tx, ownKeyHalf, hashedPubKey } = record

            pubKeyToPeerId(ack.responseSigningParty, (err, peerId) => {
                if (decode(peerId.toBytes()).digest.compare(hashedPubKey) !== 0)
                    throw Error('General error.')

                tx.decrypt(hash(bufferXOR(ownKeyHalf, ack.key)))

                const channelId = tx.getChannelId(node.peerInfo.id)

                node.paymentChannels.getChannel(channelId, (err, record) => {
                    if (!record)
                        throw Error('General error.')

                    node.paymentChannels.setChannel({
                        // index: tx.index, TODO
                        tx: tx
                    }, channelId, (err) => node.paymentChannels.getChannel(channelId, (err, record) => {
                        // log(node.peerInfo.id, `Acknowledged ${(new BN(record.tx.value)).toString()}`)
                    }))
                })
            })
        })
    })))