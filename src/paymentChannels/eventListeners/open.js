'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { log } = require('../../utils')

const VALUE_LENGTH = 32

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

    await self.setState(channelId, {
        state: self.TransactionRecordState.OPEN,
        currentIndex: state.restoreTransaction.index,
        initialValue: state.restoreTransaction.value,
        currentOffchainBalance: new BN(event.returnValues.amountA).toBuffer('be', VALUE_LENGTH),
        currentOnchainBalance: new BN(event.returnValues.amountA).toBuffer('be', VALUE_LENGTH),
        totalBalance: new BN(event.returnValues.amount).toBuffer('be', VALUE_LENGTH),
        lastTransaction: state.restoreTransaction
    })

    log(
        self.node.peerInfo.id,
        `Opened payment channel ${chalk.yellow(channelId.toString('hex'))} with txHash ${chalk.green(event.transactionHash)}. Nonce is now ${chalk.red(
            self.nonce - 1
        )}.`
    )

    self.emitOpened(channelId)
}
