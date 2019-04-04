'use strict'

const BN = require('bn.js')

const { isPartyA, pubKeyToEthereumAddress, mineBlock, log } = require('../../utils')
const { NETWORK } = require('../../constants')
const Transaction = require('../../transaction')

module.exports = (self) => async (err, event) => {
    if (err) {
        console.log(err)
        return
    }

    const channelId = Buffer.from(event.raw.topics[1].slice(2), 'hex')
    const amountA = new BN(event.returnValues.amountA)

    let tx, restoreTx
    try {
        tx = Transaction.fromBuffer(await self.node.db.get(self.Transaction(channelId)))
        restoreTx = Transaction.fromBuffer(await self.node.db.get(self.RestoreTransaction(channelId)))
    } catch (err) {
        console.log(err)
        return
    }

    const counterparty = restoreTx.counterparty
    const partyA = isPartyA(
        pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
        pubKeyToEthereumAddress(counterparty)
    )

    if (
        Buffer.from(event.returnValues.index.replace(/0x/, ''), 'hex').compare(tx.index) === -1 &&
        (partyA ? new BN(tx.value).gt(amountA) : amountA.gt(new BN(tx.value)))
    ) {
        log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

        self.registerSettlementListener(channelId)
        self.requestClose(channelId)

        return
    }

    const channel = await self.contract.methods.channels(channelId).call({
        from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
    }, 'latest')

    await new Promise((resolve, reject) => {
        const subscription = self.web3.eth.subscribe('newBlockHeaders').on('data', (block) => {
            log(self.node.peerInfo.id, `Waiting ... Block ${block.number}.`)

            if (block.timestamp > parseInt(channel.settleTimestamp)) {
                subscription.unsubscribe((err, ok) => {
                    if (err)
                        return reject(err)

                    if (ok)
                        resolve()
                })
            } else if (NETWORK === 'ganache') {
                // ================ Only for testing ================
                mineBlock(self.contract.currentProvider)
                // ==================================================
            }
        })

        if (NETWORK === 'ganache') {
            // ================ Only for testing ================
            mineBlock(self.contract.currentProvider)
            // ==================================================
        }
    })

    let receipt
    if (!self.closingRequests.has(channelId.toString('base64'))) {
        self.closingRequests.delete(channelId.toString('base64'))

        try {
            receipt = await self.contractCall(self.contract.methods.withdraw(pubKeyToEthereumAddress(counterparty)))
        } catch (err) {
            console.log(err)
            return
        }
    }

    const initialValue = new BN(restoreTx.value)
    const receivedMoney = partyA ? amountA.isub(initialValue) : initialValue.isub(amountA)

    log(self.node.peerInfo.id, `Closed payment channel \x1b[33m${channelId.toString('hex')}\x1b[0m and ${receivedMoney.isNeg() ? 'spent' : 'received'} \x1b[35m${receivedMoney.abs().toString()} wei\x1b[0m. ${receipt ? ` TxHash \x1b[32m${receipt.transactionHash}\x1b[0m.` : ''}`)

    self.node.db.batch()
        .del(self.Transaction(channelId))
        .del(self.RestoreTransaction(channelId))
        .del(self.Index(channelId))
        .del(self.ChannelKey(channelId))
        .del(self.CurrentValue(channelId))
        .del(self.TotalBalance(channelId))
        .write()

    self.emit(`closed ${channelId.toString('base64')}`, receivedMoney)
}