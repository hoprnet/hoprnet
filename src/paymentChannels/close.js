'use strict'

const { waterfall } = require('async')

module.exports = (self) => (err, event) => waterfall([
    (cb) => {
        if (err) { throw err }
        const lastTransaction = self.get(event.channelId)

        if (
            parseInt(event.index) < parseInt(lastTransaction.index)
            && null /*better Transation */) {
            self.contract.methods
                .settle(
                    lastTransaction.to,
                    lastTransaction.index,
                    lastTransaction.signature.slice(0, 32),
                    lastTransaction.slice(32))
                .send({
                    from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
                    gas: 250333, // arbitrary
                    gasPrice: '30000000000000'
                }, cb)
        } else {
            cb(null, null)
        }
    },
    (receipt, cb) => {
        self.delete(channelId)
    }
])
