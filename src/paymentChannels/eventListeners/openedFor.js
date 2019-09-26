'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { log, getId } = require('../../utils')

const Transaction = require('../../transaction')

module.exports = self => async (err, event) => {
    if (err) {
        console.log(err)
        return
    }

    const channelId = getId(event.returnValues.partyA, event.returnValues.partyB)

    log(self.node.peerInfo.id, `Received OpenedFor event for channel ${chalk.yellow(channelId.toString('hex'))}`)

    // Check that there is no record in the database
    await self
        .state(channelId)
        .then(_ => {
            throw Error(`Found record for channel ${chalk.yellow(channelId.toString())} but there should not be any record in the database.`)
        })
        .catch(err => {
            if (!err.notFound) throw err
        })

    const state = {
        state: self.TransactionRecordState.PRE_OPENED,
        currentIndex: new BN(1).toBuffer('be', Transaction.INDEX_LENGTH),
        initialValue: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        currentOffchainBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        currentOnchainBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
    }

    self.emitOpened(channelId, state)
}
