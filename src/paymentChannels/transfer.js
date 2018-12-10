'use strict'

const { waterfall } = require('async')
const PeerInfo = require('peer-info')

const { isPartyA, getId, pubKeyToEthereumAddress } = require('../utils')

module.exports = (self) => (amount, to, cb) => waterfall([
    (cb) => PeerInfo.create(to, cb),
    (toPeerInfo, cb) => self.node.getPubKey(toPeerInfo, cb),
    (toPeerInfo, cb) => {
        to = toPeerInfo.id
        const channelId = getId(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(toPeerInfo.id.pubKey.marshal()))

        if (self.has(channelId)) {
            cb(null, self.get(channelId), null)
        } else {
            self.open(toPeerInfo, cb)
        }
    },
    (lastTransaction, receipt, cb) => {
        if (receipt)
            console.log(receipt)

        if (isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), 
            pubKeyToEthereumAddress(to.pubKey.marshal()))) {
            lastTransaction.value = lastTransaction.value + amount
        } else {
            lastTransaction.value = lastTransaction.value - amount
        }
        lastTransaction.index = lastTransaction.index + 1
        lastTransaction.sign(self.node.peerInfo.id.privKey.marshal())

        self.set(lastTransaction)

        cb(null, lastTransaction)
    }
], cb)