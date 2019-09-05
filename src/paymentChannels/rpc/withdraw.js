const BN = require('bn.js')

const { pubKeyToEthereumAddress, mineBlock } = require('../../utils')
const Transaction = require('../../transaction')

/**
 * Returns a promise that resolves just when the funds from the channel are withdrawn.
 *
 * @notice When using this method with `process.env.NETWORK === 'ganache'`, this method
 * will ask Ganache to mine blocks and increase the block time until the payment channel
 * becomes withdrawable.
 *
 * @param {Buffer} channelId ID of the channel
 */
module.exports = self => channelId => {
    /**
     * Submits a withdraw transaction and cleans up attached event listeners.
     */
    const withdraw = async () => {
        const restoreTx = Transaction.fromBuffer(await self.node.db.get(self.RestoreTransaction(channelId)))

        return self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(restoreTx.counterparty))).then(receipt => {
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

            self.deleteChannel(channelId)

            return receipt
        })
    }

    /**
     * Returns a promise that returns just when the channel is withdrawable.
     */
    const waitUntilChannelIsWithdrawable = () => {
        return new Promise(async (resolve, reject) => {
            const [channel, blockTimestamp] = await Promise.all([
                self.contract.methods.channels(channelId).call(
                    {
                        from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
                    },
                    'latest'
                ),
                self.web3.eth.getBlock('latest', false).then(block => new BN(block.timestamp))
            ])

            if (channel.state == CHANNEL_STATE_WITHDRAWABLE && blockTimestamp.gt(new BN(channel.settleTimestamp))) return resolve()

            self.settleTimestamps.set(channelId.toString('hex'), new BN(channel.settleTimestamp))
            const subscription = self.web3.eth
                .subscribe('newBlockHeaders')
                .on('error', err => reject(err))
                .on('data', block => {
                    const blockTimestamp = new BN(block.timestamp)
                    log(self.node.peerInfo.id, `Waiting ... Block ${block.number}.`)

                    if (blockTimestamp.gt(self.settleTimestamps.get(channelId.toString('hex')))) {
                        subscription.unsubscribe((err, ok) => {
                            if (err) return reject(err)

                            if (ok) resolve()
                        })
                    } else if (process.env.NETWORK === 'ganache') {
                        // ================ Only for testing ================
                        mineBlock(self.contract.currentProvider)
                        // ==================================================
                    }
                })

            self.subscriptions.set(channelId.toString('hex'), subscription)
            if (process.env.NETWORK === 'ganache') {
                // ================ Only for testing ================
                mineBlock(self.contract.currentProvider)
                // ==================================================
            }
        })
    }

    return waitUntilChannelIsWithdrawable()
        .then(() => withdraw())
        .then(() => self.node.db.get(self.CurrentOnChainBalance(channelId)))
        .then(balance => new BN(balance))
}
