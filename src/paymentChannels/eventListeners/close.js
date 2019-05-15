'use strict'

const BN = require('bn.js')
const chalk = require('chalk')

const { isPartyA, pubKeyToEthereumAddress, mineBlock, log } = require('../../utils')
const Transaction = require('../../transaction')

module.exports = (self) => {
    /**
     * Checks whether there is a restore transaction in the database. Otherwise search for
     * a stashed restore transaction and return that one instead, otherwise throw an error.
     * 
     * @param {Buffer} channelId ID of the payment channel
     */
    function getRestoreTransaction(channelId) {
        return self.node.db.get(self.RestoreTransaction(channelId))
            .then((tx) => Transaction.fromBuffer(tx))
            .catch((err) => {
                if (!err.notFound)
                    throw err

                return self.node.db.get(self.StashedRestoreTransaction(channelId))
                    .then((tx) => Transaction.fromBuffer(tx))
            })

    }

    function recoverCounterparty(transactionHash, channelId) {
        return getRestoreTransaction(channelId)
            .then((tx) => tx.counterparty)
            .catch((err) => {
                if (!err.notFound)
                    console.log(err.message)

                self.web3.getTransaction(transactionHash)
                    .then((closingTx) => closingTx.from)
            })
    }

    /**
     * Returns a promise that resolves just when the payment channel is withdrawable.
     * 
     * @notice When using this method with `NETWORK === 'ganache'`, this method will ask
     * testnet to mine blocks and increase the block time until the payment channel becomes
     * withdrawable.
     *  
     * @param {String} timestamp the timestamp when the payment channel is withdrawable.
     */
    function waitUntilChannelIsWithdrawable(timestamp) {
        return new Promise((resolve, reject) => {
            const subscription = self.web3.eth.subscribe('newBlockHeaders').on('data', (block) => {
                log(self.node.peerInfo.id, `Waiting ... Block ${block.number}.`)

                if (block.timestamp > parseInt(timestamp)) {
                    subscription.unsubscribe((err, ok) => {
                        if (err)
                            return reject(err)

                        if (ok)
                            resolve()
                    })
                } else if (process.env.NETWORK === 'ganache') {
                    // ================ Only for testing ================
                    mineBlock(self.contract.currentProvider)
                    // ==================================================
                }
            })

            if (process.env.NETWORK === 'ganache') {
                // ================ Only for testing ================
                mineBlock(self.contract.currentProvider)
                // ==================================================
            }
        })
    }

    /**
     * Withdraws funds from a payment channel.
     * 
     * @param {Buffer} channelId ID of the channel
     * @param {Buffer} counterparty public key of the counterparty
     */
    async function withdrawFunds(channelId, counterparty) {
        const channel = await self.contract.methods.channels(channelId).call({
            from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
        }, 'latest')

        await waitUntilChannelIsWithdrawable(channel.settleTimestamp)

        self.closingRequests.delete(channelId.toString('base64'))

        return self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(counterparty)))
    }

    async function emitEvent(channelId, receivedMoney, receipt = null) {
        log(self.node.peerInfo.id, `Closed payment channel ${chalk.yellow(channelId.toString('hex'))} and ${receivedMoney.isNeg() ? 'spent' : 'received'} ${chalk.magenta(receivedMoney.abs().toString())} wei\x1b[0m. ${receipt ? ` TxHash \x1b[32m${receipt.transactionHash}\x1b[0m.` : ''}`)

        await self.deleteChannel(channelId)

        self.emitClosed(channelId, receivedMoney)
    }

    async function getReceivedMoney(amountA, channelId, isPartyA) {
        const initialValue = new BN(await self.node.db.get(self.InitialValue(channelId)))

        return isPartyA ? amountA.isub(initialValue) : initialValue.isub(amountA)
    }

    return async (err, event) => {
        if (err) {
            console.log(err)
            return
        }

        const channelId = Buffer.from(event.raw.topics[1].slice(2), 'hex')
        const amountA = new BN(event.returnValues.amountA)
        const counterparty = await recoverCounterparty(event.transactionHash, channelId)

        const partyA = isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            pubKeyToEthereumAddress(counterparty)
        )

        try {
            const tx = Transaction.fromBuffer(await self.node.db.get(self.Transaction(channelId)))

            const eventIndex = new BN(event.returnValues.index.replace(/0x/, ''), 16)
            const txIndex = new BN(tx.index)

            if (eventIndex.lt(txIndex) && (partyA ? new BN(tx.value).gt(amountA) : amountA.gt(new BN(tx.value)))) {
                log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

                self.registerSettlementListener(channelId)
                self.requestClose(channelId)

                return
            }
        } catch (err) {
            if (!err.notFound) {
                console.log(err)
                return
            }
        }

        let receipt
        if (!self.closingRequests.has(channelId.toString('base64'))) {
            try {
                receipt = await withdrawFunds(channelId, counterparty)
            } catch (err) {
                console.log(err.message)
                return
            }
        }

        emitEvent(channelId, await getReceivedMoney(amountA, channelId, partyA), receipt)
    }
}