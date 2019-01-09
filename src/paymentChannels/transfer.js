'use strict'

const { waterfall } = require('neo-async')
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
            cb(null, self.get(channelId))
        } else {
            self.open(toPeerInfo, cb)
        }
    },
    (lastTx, cb) => {
        if (isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()), 
            pubKeyToEthereumAddress(to.pubKey.marshal()))) {
            lastTx.value = lastTx.value - amount
        } else {
            lastTx.value = lastTx.value + amount
        }
        lastTx.index = lastTx.index + 1
        lastTx.sign(self.node.peerInfo.id.privKey.marshal())

        self.set(lastTx)

        cb(null, lastTx)
    }
], cb)