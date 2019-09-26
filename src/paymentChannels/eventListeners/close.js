'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { isPartyA, pubKeyToEthereumAddress, log } = require('../../utils')

const Transaction = require('../../transaction')

module.exports = self => {
    /**
     * Checks whether the previously published transaction is the most profitable transaction for
     * this party.
     * Returns `true` if there is a better on, otherwise `false`
     * @param {BN} newBalance current onchain balance of payment channel
     * @param {BN} newIndex current onchain index of payment channel
     * @param {Object} state current offchain state from database
     * @param {boolean} partyA must be true iff this node is PartyA of the payment channel
     */
    const hasBetterTransaction = (newBalance, newIndex, state, partyA) => {
        if (!state.lastTransaction) return false

        return (
            newIndex.lt(new BN(state.lastTransaction.index)) &&
            (partyA ? new BN(state.lastTransaction.value).gt(newBalance) : newBalance.gt(new BN(state.lastTransaction.value)))
        )
    }

    const listener = async (err, event) => {
        if (err) {
            console.log(err)
            return
        }

        const channelId = Buffer.from(event.returnValues.channelId.replace(/0x/, ''), 'hex')

        let state
        try {
            state = await self.state(channelId)
        } catch (err) {
            if (err.notFound) {
                log(self.node.peerInfo.id, `Listening to the closing event of channel ${chalk.yellow(channelId.toString('hex'))} but there is no record in the database.`)
                return
            }

            throw err
        }

        const partyA = isPartyA(
            /* prettier-ignore */
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(state.counterparty)
        )

        state.currentOnchainBalance = new BN(event.returnValues.amountA).toBuffer('be', Transaction.VALUE_LENGTH)
        state.currentIndex = event.returnValues.index

        if (hasBetterTransaction(new BN(event.returnValues.amountA), new BN(event.returnValues.index, 'hex'), state, partyA)) {
            log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

            // @TODO database might be outdated when the event comes back

            state.state = self.TransactionRecordState.SETTLING

            await self.setState(channelId, state)

            self.registerSettlementListener(channelId)

            self.submitSettlementTransaction(channelId, state.lastTransaction)
        } else {
            state.state = self.TransactionRecordState.SETTLED

            self.closingSubscriptions.get(channelId.toString('hex')).unsubscribe()

            const networkState = self.contract.methods.channels(channelId).call(
                {
                    from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
                },
                'latest'
            )

            self.settleTimestamps.set(channelId.toString('hex'), new BN(networkState.settleTimestamp))
            self.emitClosed(channelId, state)
        }
    }

    return listener
}
