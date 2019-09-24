'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { log } = require('../../utils')

const Transaction = require('../../transaction')

module.exports = self => async (err, event) => {
    if (err) {
        console.log(err)
        return
    }

    const channelId = Buffer.from(event.returnValues.channelId.replace(/0x/, ''), 'hex')

    let state
    try {
        state = await self.state(channelId)
    } catch (err) {
        if (err.notFound)
            throw Error(`${chalk.blue(self.node.peerInfo.id.toB58String())}: Opening request of channel ${chalk.yellow(channelId.toString('hex'))} not found.`)

        throw err
    }

    // @TODO check incoming event for plausibility

    Object.assign(state, {
        state: self.TransactionRecordState.OPEN,
        currentIndex: state.restoreTransaction.index,
        initialValue: state.restoreTransaction.value,
        currentOffchainBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        currentOnchainBalance: new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH),
        totalBalance: new BN(event.returnValues.amount).toBuffer('be', Transaction.VALUE_LENGTH),
        lastTransaction: state.restoreTransaction
    })

    log(
        self.node.peerInfo.id,
        `Opened payment channel ${chalk.yellow(channelId.toString('hex'))} with txHash ${chalk.green(event.transactionHash)}. Nonce is now ${chalk.cyan(
            self.nonce
        )}.`
    )

    self.emitOpened(channelId, state)
}
