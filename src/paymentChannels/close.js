'use strict'

const { waterfall } = require('async')

module.exports = (self) => (err, event) => {
    if (err) { throw err }

    const channelId = Buffer.from(event.returnValues.channelId.slice(2), 'hex')

    if (self.has(channelId)) {
        const lastTransaction = self.get(Buffer.from(event.returnValues.channelId.slice(2), 'hex'))

        if (
            parseInt(event.returnValues.index) < parseInt(lastTransaction.index)
            && null /* better Transation */) {
            self.settle(lastTransaction.channelId, (err) => {
                if (err) { throw err }
    
                self.delete(lastTransaction.channelId)
            })
        } else {
            self.delete(lastTransaction.channelId)
        }
    }
}