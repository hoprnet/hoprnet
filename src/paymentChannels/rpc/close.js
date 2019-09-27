'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { pubKeyToEthereumAddress, log, bufferToNumber } = require('../../utils')

const Transaction = require('../../transaction')

const { ChannelState } = require('../enums.json')

const OPENING_TIMEOUT = 60 * 1000

module.exports = self => {
    /**
     * Checks whether the counterparty may has a more recent transaction.
     *
     * @param {TransactionRecord}
     */
    const counterpartyHasMoreRecentTransaction = record => {
        if (!record.lastTransaction) return true

        return new BN(record.currentIndex).gt(new BN(record.lastTransaction.index))
    }

    /**
     * Returns a promise that resolves just when a settlement transaction were successfully
     * submitted to the Ethereum network.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Transaction} [tx] tx that is used to close the payment channel
     */
    const submitSettlementTransaction = async (channelId, localState) => {
        log(self.node.peerInfo.id, `Trying to close payment channel ${chalk.yellow(channelId.toString('hex'))}. Nonce is ${chalk.cyan(self.nonce)}`)

        await self.setState(channelId, {
            state: self.TransactionRecordState.SETTLING
        })

        let receipt
        try {
            receipt = await self.contractCall(
                self.contract.methods.closeChannel(
                    localState.lastTransaction.index,
                    localState.lastTransaction.nonce,
                    new BN(localState.lastTransaction.value).toString(),
                    localState.lastTransaction.curvePoint.slice(0, 32),
                    localState.lastTransaction.curvePoint.slice(32, 33),
                    localState.lastTransaction.signature.slice(0, 32),
                    localState.lastTransaction.signature.slice(32, 64),
                    bufferToNumber(localState.lastTransaction.recovery) + 27
                )
            )

            log(
                self.node.peerInfo.id,
                /* prettier-ignore */
                `Settled channel ${chalk.yellow(channelId.toString('hex'))} with txHash ${chalk.green(receipt.transactionHash)}. Nonce is now ${chalk.cyan(self.nonce)}.`
            )
        } catch (err) {
            const networkState = await self.contractCall(self.contractCall.methods.channels(channelId))
            console.log(`Couldn't close channel ${chalk.yellow(channelId.toString('hex'))}. On-chain state is ${JSON.stringify(networkState)}`)
        }
    }

    const initiateClosing = async (channelId, localState, networkState) => {
        switch (parseInt(networkState.state)) {
            case ChannelState.UNINITIALIZED:
                log(self.node.peerInfo.id, `Channel ${chalk.yellow(channelId.toString('hex'))} doesn't exist.`)
                await self.deleteState(channelId)
                return new BN(0)
            case ChannelState.ACTIVE:
                if (counterpartyHasMoreRecentTransaction(localState)) {
                    let lastTx
                    try {
                        lastTx = await self.getLatestTransactionFromCounterparty(channelId, localState)
                    } catch (err) {
                        console.log(err)
                    }

                    // @TODO take the received transaction only if it is more profitable than the previous one
                    if (new BN(lastTx.index).gt(new BN(localState.lastTransaction.index))) {
                        localState.lastTransaction = lastTx
                    }
                }

                return new Promise(resolve => {
                    self.onceClosed(channelId, newState => {
                        Object.assign(localState, newState)

                        resolve(self.withdraw(channelId, localState, networkState))
                    })

                    submitSettlementTransaction(channelId, localState)
                })
            case ChannelState.PENDING_SETTLEMENT:
                return self.withdraw(channelId, localState, networkState)
            default:
                log(self.node.peerInfo.id, `Channel in unknown state: channel.state = ${chalk.red(channel.state)}.`)

                return new BN(0)
        }
    }

    /**
     * Manages the settlement of a payment channel.
     *
     * @notice Resolves once the payment channel is withdrawn.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Object} [state] current off-chain state
     */
    const close = (channelId, state) => new Promise(async (resolve, reject) => {
        if (!state) {
            try {
                state = await self.state(channelId)
            } catch (err) {
                return reject(err)
            }

        }

        const networkState = await self.contract.methods.channels(channelId).call(
            {
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            },
            'latest'
        )

        if (state.preOpened && !state.lastTransaction && !state.restoreTransaction)
            return reject(Error(
                `Cannot close channel ${chalk.yellow(channelId.toString('hex'))} because it was opened by a third party and the counterparty is still unknown.`
            ))

        switch (state.state) {
            case self.TransactionRecordState.OPENING:
                // This can only happen if the node which initiated the opening procedure
                // failed while doing that
                const timeout = setTimeout(() => {
                    console.log(`Could not close channel ${chalk.yellow(channelId.toString('hex'))} because no one opened it within the timeout.`)
                    return resolve(self.deleteState(channelId).then(() => new BN(0)))
                }, OPENING_TIMEOUT)

                self.onceOpened(channelId, newState => {
                    clearTimeout(timeout)
                    networkState.state = ChannelState.ACTIVE
                    resolve(initiateClosing(channelId, newState, networkState))
                })
                break
            case self.TransactionRecordState.PRE_OPENED:
            case self.TransactionRecordState.OPEN:
                return resolve(initiateClosing(channelId, state, networkState))
            case self.TransactionRecordState.SETTLED:
            case self.TransactionRecordState.WITHDRAWABLE:
            case self.TransactionRecordState.WITHDRAWING:
                state.currentOnchainBalance = new BN(networkState.balanceA, 'hex').toBuffer('be', Transaction.VALUE_LENGTH)
                state.currentIndex = networkState.index

                // @TODO insert currect receivedMoney
                return resolve(self.withdraw(channelId, state, networkState).then(_ => new BN(0)))
            case self.TransactionRecordState.SETTLING:
                log(self.node.peerInfo.id, `Channel ${chalk.yellow(channelId.toString('hex'))} is already settling. No action required.`)
                return resolve(new BN(0))
            default:
                return reject(Error(`Channel is in state ${state.state}`))
        }
    })

    return close
}
