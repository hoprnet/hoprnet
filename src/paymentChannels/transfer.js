'use strict'

const { waterfall } = require('neo-async')
const PeerInfo = require('peer-info')
const { BN } = require('web3').utils

const { isPartyA, getId, pubKeyToEthereumAddress, bufferToNumber, numberToBuffer } = require('../utils')

const { INDEX_LENGTH, VALUE_LENGTH } = require('../transaction')

module.exports = (self) => (amount, to, cb) => waterfall([
    (cb) => PeerInfo.create(to, cb),
    (toPeerInfo, cb) => self.node.getPubKey(toPeerInfo, cb),
    (toPeerInfo, cb) => {
        to = toPeerInfo.id

        const channelId = getId(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal()))

        self.getChannel(channelId, cb)
    },
    (record, cb) => {
        if (typeof record === 'function') {
            cb = record
            record = null
        }

        if (record) {
            cb(null, record.tx)
        } else {
            self.open(to, cb)
        }
    },
    (lastTx, cb) => {
        // channelId is computed by recovering the public key from a signature,
        // so it'll change when transaction properties change!
        const channelId = lastTx.getChannelId(self.node.peerInfo.id)

        const lastValue = new BN(lastTx.value)

        if (isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(to.pubKey.marshal()))) {
            lastTx.value = lastValue.sub(amount).toBuffer('be', VALUE_LENGTH)
        } else {
            lastTx.value = lastValue.add(amount).toBuffer('be', VALUE_LENGTH)
        }

        lastTx.index = numberToBuffer(bufferToNumber(lastTx.index) + 1, INDEX_LENGTH)
        lastTx.sign(self.node.peerInfo.id)

        self.setChannel({
            index: lastTx.index
        }, channelId, (err) => {
            if (err)
                throw err

            cb(null, lastTx)
        })
    }
], cb)