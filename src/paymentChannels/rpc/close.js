'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { pubKeyToEthereumAddress, log, bufferToNumber } = require('../../utils')

const Transaction = require('../../transaction')

const SETTLEMENT_TIMEOUT = 40000

const CHANNEL_STATE_UNINITIALIZED = 0
const CHANNEL_STATE_FUNDED = 3
const CHANNEL_STATE_WITHDRAWABLE = 4

module.exports = self => {
    /**
     * Checks whether the counterparty may has a more recent transaction.
     *
     * @param {TransactionRecord}
     */
    const counterpartyHasMoreRecentTransaction = record => {
        return new BN(record.currentIndex).gt(new BN(record.lastTransaction.index))
    }

    /**
     * Returns a promise that resolves just when a settlement transaction were successfully
     * submitted to the Ethereum network.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Transaction} [tx] tx that is used to close the payment channel
     */
    const submitSettlementTransaction = async (channelId, tx, channelKey) => {
        log(self.node.peerInfo.id, `Trying to close payment channel ${chalk.yellow(channelId.toString('hex'))}. Nonce is ${chalk.cyan(self.nonce)}`)

        const receipt = await self.contractCall(
            self.contract.methods.closeChannel(
                tx.index,
                tx.nonce,
                new BN(tx.value).toString(),
                tx.curvePoint.slice(0, 32),
                tx.curvePoint.slice(32, 33),
                tx.signature.slice(0, 32),
                tx.signature.slice(32, 64),
                bufferToNumber(tx.recovery) + 27
            )
        )

        log(
            self.node.peerInfo.id,
            /* prettier-ignore */
            `Settled channel ${chalk.yellow(channelId.toString('hex'))} with txHash ${chalk.green(receipt.transactionHash)}. Nonce is now ${chalk.cyan(self.nonce)}.`
        )
        return receipt
    }

    const initiateClosing = async (channelId, localState, networkState) => {
        switch (parseInt(networkState.state)) {
            case CHANNEL_STATE_UNINITIALIZED:
                log(self.node.peerInfo.id, `Channel ${chalk.yellow(channelId.toString('hex'))} doesn't exist.`)
                await self.deleteState(channelId)
                return new BN(0)
            case CHANNEL_STATE_FUNDED:
                let lastTx
                if (counterpartyHasMoreRecentTransaction(localState)) {
                    lastTx = await new Promise(async resolve => {
                        const timeout = setTimeout(resolve, SETTLEMENT_TIMEOUT)

                        try {
                            resolve(
                                self
                                    .getLatestTransactionFromCounterparty({
                                        channelId,
                                        state: localState
                                    })
                                    .then(results => {
                                        results = results.filter(result => result.channelId.equals(channelId))

                                        if (!results) throw Error('Got no response from counterparty')

                                        return results.transaction
                                    })
                            )
                        } catch (err) {
                            console.log(err.message)
                            resolve()
                        } finally {
                            clearTimeout(timeout)
                        }
                    })
                } else {
                    lastTx = localState.lastTransaction
                }

                return new Promise(resolve => {
                    self.onceClosed(
                        channelId,
                        (() => {
                            submitSettlementTransaction(channelId, lastTx)

                            return () => {
                                resolve(self.withdraw(channelId, localState, networkState))
                            }
                        })()
                    )
                })
            case CHANNEL_STATE_WITHDRAWABLE:
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
    const close = async (channelId, state) => {
        if (!state) state = await self.state(channelId)

        const networkState = await self.contract.methods.channels(channelId).call(
            {
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            },
            'latest'
        )

        switch (state.state) {
            case self.TransactionRecordState.OPENING:
                const timeout = setTimeout(() => {
                    throw Error(`Could not close channel ${chalk.yellow(channelId.toString('hex'))} because no one opened it within the timeout.`)
                })

                return new Promise(resolve => {
                    self.onceOpened(channelId, () => {
                        clearTimeout(timeout)
                        networkState.state = CHANNEL_STATE_FUNDED
                        resolve(initiateClosing(channelId, state, networkState))
                    })
                })
            case self.TransactionRecordState.OPEN:
                return initiateClosing(channelId, state, networkState)
            default:
                throw Error('TODO')
        }
    }

    return close
}
