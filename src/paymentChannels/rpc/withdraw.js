const BN = require('bn.js')

const chalk = require('chalk')

const { pubKeyToEthereumAddress, mineBlock, log } = require('../../utils')

const CHANNEL_STATE_WITHDRAWABLE = 4

/**
 * Returns a promise that resolves just when the funds from the channel are withdrawn.
 *
 *
 * @param {Buffer} channelId ID of the channel
 */
module.exports = self => {
    /**
     * Submits a withdraw transaction and cleans up attached event listeners.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Object} localState current off-chain state from the database
     */
    const withdraw = async (channelId, localState) => {
        const receipt = await self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(localState.restoreTransaction.counterparty)))

        const subscription = self.subscriptions.get(channelId.toString('hex'))

        if (subscription) {
            subscription.unsubscribe()
            self.subscriptions.delete(channelId.toString('hex'))
        }

        const closingSubscription = self.closingSubscriptions.get(channelId.toString('hex'))
        if (closingSubscription) {
            closingSubscription.unsubscribe()
            self.closingSubscriptions.delete(channelId.toString())
        }

        return receipt
    }

    /**
     * Returns a promise that returns just when the channel is withdrawable.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Object} networkState on-chain state of the payment channel
     */
    const waitUntilChannelIsWithdrawable = (channelId, networkState) =>
        new Promise(async (resolve, reject) => {
            const blockTimestamp = await self.web3.eth.getBlock('latest', false).then(block => new BN(block.timestamp))

            if (networkState.state == CHANNEL_STATE_WITHDRAWABLE && blockTimestamp.gt(new BN(networkState.settleTimestamp))) return resolve()

            self.settleTimestamps.set(channelId.toString('hex'), new BN(networkState.settleTimestamp))

            const subscription = self.web3.eth
                .subscribe('newBlockHeaders')
                .on('error', err => reject(err))
                .on('data', block => {
                    const blockTimestamp = new BN(block.timestamp)
                    log(self.node.peerInfo.id, `Waiting ... ${chalk.cyan(`Block ${block.number}`)}.`)

                    if (blockTimestamp.gt(self.settleTimestamps.get(channelId.toString('hex')))) {
                        subscription.unsubscribe((err, ok) => {
                            if (err) return reject(err)

                            if (ok) resolve()
                        })
                    } else if (process.env['NETWORK'] === 'ganache') {
                        // ================ Only for testing ================
                        mineBlock(self.contract.currentProvider)
                        // ==================================================
                    }
                })

            self.subscriptions.set(channelId.toString('hex'), subscription)
            if (process.env['NETWORK'] === 'ganache') {
                // ================ Only for testing ================
                mineBlock(self.contract.currentProvider)
                // ==================================================
            }
        })

    /**
     * Waits until the payment channel is withdrawable and submits a withdrawal transaction
     * it is withdrawable.
     *
     * @notice When using this method with `process.env['NETWORK'] === 'ganache'`, this method
     * will ask Ganache to mine blocks and increase the block time until the payment channel
     * becomes withdrawable.
     *
     * @param {Buffer} channelId ID of the channel
     * @param {Object} localState current state from the database
     * @param {Object} networkState current on-chain state of the payment channel
     */
    const initiateWithdrawal = async (channelId, localState, networkState) => {
        await waitUntilChannelIsWithdrawable(channelId, networkState)

        const receipt = await withdraw(channelId, localState)

        log(
            self.node.peerInfo.id,
            `Successfully submitted withdrawal transaction of channel ${chalk.yellow(channelId.toString('hex'))} with transaction ${chalk.green(
                receipt.transactionHash
            )}.`
        )

        await self.deleteState(channelId)

        return new BN(localState.currentOnchainBalance)
    }

    return initiateWithdrawal
}
