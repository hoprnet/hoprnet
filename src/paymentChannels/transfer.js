'use strict'

const { waterfall } = require('async')
const PeerInfo = require('peer-info')

const { isPartyA, getId, pubKeyToEthereumAddress } = require('../utils')

module.exports = (self) => (amount, to, cb) => waterfall([
    (cb) => {
        if (!to.pubKey) {
            waterfall([
                (cb) => PeerInfo.create(to, cb),
                (peerInfo, cb) => self.node.getPubKey(peerInfo, cb),
                (to, cb) => self.open(to, cb)
            ], cb)
        } else {
            const channelId = getId(
                pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                pubKeyToEthereumAddress(to.pubKey.marshal()))
    
            if (self.has(channelId)) {
                cb(null, self.get(channelId), null)
            } else {
                self.open(to, cb)
            }
        }
    },
    (lastTransaction, receipt, cb) => {
        if (receipt)
            console.log(receipt)

        if (isPartyA(self.node, to)) {
            lastTransaction.value = lastTransaction.value + amount
        } else {
            lastTransaction.value = lastTransaction.value - amount
        }
        lastTransaction.index = lastTransaction.index + 1

        lastTransaction.sign(self.node.peerInfo.id.privKey.marshal())

        cb(null, lastTransaction)
    }
], cb)