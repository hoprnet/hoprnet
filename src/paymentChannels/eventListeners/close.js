'use strict'

const BN = require('bn.js')

const { isPartyA, pubKeyToEthereumAddress, log } = require('../../utils')

module.exports = (self) => {
    const setCurrentOnChainBalance = async (channelId, isPartyA, newBalance) => {
        const initialBalance = new BN(await self.node.db.get(self.InitialValue(channelId)))
        const currentBalance = isPartyA ? newBalance.sub(initialBalance) : initialBalance.sub(newBalance)

        self.node.db.put(self.CurrentOnChainBalance(channelId), currentBalance.toBuffer('be', 32))
    }

    const hasBetterTransaction = async (newBalance, newIndex, transactionHash, channelId) => {
        const counterparty = await self.web3.eth.getTransaction(transactionHash)
            .then((closingTx) => closingTx.from)

        const partyA = isPartyA(
            pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal()),
            counterparty
        )

        setCurrentOnChainBalance(channelId, isPartyA, newBalance)

        const tx = await self.getLastTransaction(channelId)

        return newIndex.lt(new BN(tx.index)) && (partyA ? new BN(tx.value).gt(newBalance) : newBalance.gt(new BN(tx.value)))
    }

    return async (err, event) => {
        if (err) {
            console.log(err)
            return
        }

        const channelId = Buffer.from(event.topics[1].replace(/0x/, ''), 'hex')

        const { eventIndex, amountA } = self.web3.eth.abi.decodeParameters([{
            type: 'bytes16',
            name: 'eventIndex'
        }, {
            type: 'uint256',
            name: 'amountA'
        }], event.data)

        if (await hasBetterTransaction(new BN(amountA), new BN(eventIndex, 'hex'), event.transactionHash, channelId)) {
            log(self.node.peerInfo.id, `Found better transaction for payment channel ${channelId.toString('hex')}.`)

            self.submitSettlementTransaction(channelId)
        } else {
            const channel = await self.contract.methods.channels(channelId).call({
                from: pubKeyToEthereumAddress(self.node.peerInfo.id.pubKey.marshal())
            }, 'latest')

            self.settleTimestamps.set(channelId.toString('hex'), new BN(channel.settleTimestamp))
            self.emitClosed(channelId)
        }
    }
}