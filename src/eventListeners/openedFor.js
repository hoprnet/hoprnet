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
    const record = await self.state(channelId).catch(err => {
        if (!err.notFound) throw err
    })

    if (record) {
        // There should be no entry in the database.
        // @TODO remove closed channels
        // switch (record.state) {
        //     case self.TransactionRecordState.INITIALIZED:
        //     case self.TransactionRecordState.OPENING:
        //     case self.TransactionRecordState.PRE_OPENED:
        //     case self.TransactionRecordState.OPEN:
        //     case self.TransactionRecordState.SETTLING:
        //     case self.TransactionRecordState.SETTLED:
        //     case self.TransactionRecordState.WITHDRAWABLE:
        //     case self.TransactionRecordState.WITHDRAWING:
        //     case self.TransactionRecordState.WITHDRAWN:
        //     default:
        //         throw Error(
        //             `Found record for channel ${chalk.yellow(
        //                 channelId.toString('hex')
        //             )} but there should not be any record in the database. Channel seems to be in state '${record.state}'`
        //         )
        // }
        log(
            self.node.peerInfo.id,
            `Found record for channel ${chalk.yellow(
                channelId.toString('hex')
            )} but there should not be any record in the database. Channel seems to be in state '${record.state}'`
        )
        await self.deleteState(channelId)
    }

    const state = {
        state: self.TransactionRecordState.PRE_OPENED,
        currentIndex: new BN(1).toBuffer('be', Transaction.INDEX_LENGTH),
        initialBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        currentOffchainBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        currentOnchainBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        totalBalance: new BN(event.returnValues.amount).toBuffer('be', Transaction.VALUE_LENGTH),
        preOpened: true
    }

    self.registerSettlementListener(channelId)

    self.emitOpened(channelId, state)
}
