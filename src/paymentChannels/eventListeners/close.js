'use strict'

const BN = require('bn.js')

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
        return new Promise((resolve, reject) =>
            self.node.db.get(self.RestoreTransaction(channelId))
                .then((tx) => resolve(Transaction.fromBuffer(tx)))
                .catch((err) => {
                    if (!err.notFound)
                        return reject(err)

                    self.node.db.get(self.StashedRestoreTransaction(channelId))
                        .then((tx) => resolve(Transaction.fromBuffer(tx)))
                        .catch(reject)
                })

        )
    }

    function recoverCounterparty(transactionHash, channelId) {
        return new Promise((resolve, reject) => {
            getRestoreTransaction(channelId)
                .then((tx) => resolve(tx.counterparty))
                .catch((err) => {
                    if (!err.notFound)
                        console.log(err.message)

                    self.web3.getTransaction(transactionHash)
                        .then((closingTx) => resolve(closingTx.from))
                        .catch(reject)
                })
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

    async function emitEvent(channelId, receivedMoney, receipt = null) {
        log(self.node.peerInfo.id, `Closed payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m and ${receivedMoney.isNeg() ? 'spent' : 'received'} \x1b[35m${receivedMoney.abs().toString()} wei\x1b[0m. ${receipt ? ` TxHash \x1b[32m${receipt.transactionHash}\x1b[0m.` : ''}`)

        await self.deleteChannel(channelId)

        self.emit(`closed ${channelId.toString('base64')}`, receivedMoney)
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

        let tx
        try {
            tx = Transaction.fromBuffer(await self.node.db.get(self.Transaction(channelId)))
        } catch (err) {
            if (!err.notFound) 
                console.log(err)

            const receivedMoney = await getReceivedMoney(amountA, channelId, partyA) 
            emitEvent(channelId, receivedMoney)
            return
        } 

        const eventIndex = new BN(event.returnValues.index.replace(/0x/, ''), 16)
        const txIndex = new BN(tx.index)

        if (eventIndex.lt(txIndex) && (partyA ? new BN(tx.value).gt(amountA) : amountA.gt(new BN(tx.value)))) {
            log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

            self.registerSettlementListener(channelId)
            self.requestClose(channelId)

            return
        }

        let receipt
        if (!self.closingRequests.has(channelId.toString('base64'))) {
            const channel = await self.contract.methods.channels(channelId).call({
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            }, 'latest')

            await waitUntilChannelIsWithdrawable(channel.settleTimestamp)

            self.closingRequests.delete(channelId.toString('base64'))

            try {
                receipt = await self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(counterparty)))
            } catch (err) {
                console.log(err)
                return
            }
        }

        emitEvent(channelId, await getReceivedMoney(amountA, channelId, partyA), receipt)
    }
}