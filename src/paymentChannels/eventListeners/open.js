'use strict'

const { log } = require('../../utils')

module.exports = (self) => (err, event) => {
    if (err)
        throw err

    const channelId = Buffer.from(event.raw.topics[1].slice(2), 'hex')
    const record = self.openingRequests.get(channelId.toString('base64'))

    if (!record)
        return

    self.setChannel(record, { sync: true }, (err) => {
        if (err) {
            console.log(err)
        }

        log(self.node.peerInfo.id, `Opened payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m with txHash \x1b[32m${event.transactionHash}\x1b[0m. Nonce is now \x1b[31m${self.nonce - 1}\x1b[0m.`)

        self.openingRequests.delete(channelId.toString('base64'))
        self.emit(`opened ${channelId.toString('base64')}`, null, record)
    })
}