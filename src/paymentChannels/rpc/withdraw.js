const BN = require('bn.js')

const chalk = require('chalk')

const { pubKeyToPeerId, pubKeyToEthereumAddress, mineBlock, log } = require('../../utils')
const { ChannelState } = require('../enums.json')

module.exports = self => {
    /**
     * Submits a withdraw transaction and cleans up attached event listeners.
     *
     * @param {Buffer} channelId ID of the payment channel
     * @param {Object} localState current off-chain state from the database
     */
    const withdraw = async (channelId, localState) => {
        localState.state = self.TransactionRecordState.WITHDRAWING

        await self.setState(channelId, {
            state: self.TransactionRecordState.WITHDRAWING
        })

        let receipt
        try {
            receipt = await self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(localState.counterparty)))
        } catch (err) {
            const networkState = await self.contract.methods.channels(channelId).call({
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            })

            switch (parseInt(networkState.state)) {
                case ChannelState.UNINITIALIZED:
                    return
                // @TODO this is potentially useless
                case ChannelState.PENDING_SETTLEMENT:
                    receipt = await self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(localState.counterparty)))
                    break
                default:
                    throw Error(`Could not withdraw channel ${chalk.yellow(channelId.toString('hex'))} because its state is '${networkState.state}'.`)
            }
        }

        const subscription = self.subscriptions.get(channelId.toString('hex'))

        if (subscription) {
            subscription.unsubscribe()
            self.subscriptions.delete(channelId.toString('hex'))
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

            if (networkState.state == ChannelState.PENDING_SETTLEMENT && blockTimestamp.gt(new BN(networkState.settleTimestamp))) return resolve()

            self.settleTimestamps.set(channelId.toString('hex'), new BN(networkState.settleTimestamp))

            const subscription = self.web3.eth
                .subscribe('newBlockHeaders')
                .on('error', err => reject(err))
                .on('data', block => {
                    const blockTimestamp = new BN(block.timestamp)
                    log(
                        self.node.peerInfo.id,
                        `Waiting for timestamp ${self.settleTimestamps.get(channelId.toString('hex')).toString()} ... ${chalk.cyan(
                            `Block ${block.number}, timestamp ${blockTimestamp.toString()}`
                        )}.`
                    )

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
        if (networkState.settleTimestamp === '0') throw Error('wrong timestamp')

        if (!localState.counterparty) throw Error('no counterparty')

        await waitUntilChannelIsWithdrawable(channelId, networkState)

        const receipt = await withdraw(channelId, localState)

        if (receipt) {
            log(
                self.node.peerInfo.id,
                `Successfully submitted withdrawal transaction of channel ${chalk.yellow(channelId.toString('hex'))} with transaction ${chalk.green(
                    receipt.transactionHash
                )}. Nonce is now ${chalk.cyan(self.node.paymentChannels.nonce)}`
            )
        }

        const receivedMoney = self.getEmbeddedMoney(localState.currentOnchainBalance, localState.initialBalance, await pubKeyToPeerId(localState.counterparty))

        await self.deleteState(channelId)

        return new BN(receivedMoney)
    }

    return initiateWithdrawal
}
