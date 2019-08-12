'use strict'

const Transaction = require('../../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { sign } = require('secp256k1')
const { pubKeyToEthereumAddress, numberToBuffer, bufferToNumber, log } = require('../../utils')
const { fromWei } = require('web3-utils')
const BN = require('bn.js')

const { SIGNATURE_LENGTH } = require('../../transaction')

module.exports = node => {
    const handleOpeningRequest = async (data, cb) => {
        if (data.length !== Transaction.SIZE) {
            log(node.peerInfo.id, 'Invalid size of payment channel opening request.')
            return cb(null, Buffer.alloc(0))
        }

        const restoreTx = Transaction.fromBuffer(data)

        if (bufferToNumber(restoreTx.index) !== 1) {
            log(node.peerInfo.id, 'Invalid payment channel opening request.')
            return cb(null, Buffer.alloc(0))
        }

        const counterparty = restoreTx.counterparty
        const channelId = restoreTx.getChannelId(node.peerInfo.id)

        try {
            await node.db.get(node.paymentChannels.StashedRestoreTransaction(channelId))
            return
        } catch (err) {
            if (!err.notFound) {
                log(node.peerInfo.id, err.message)
                return cb(null, Buffer.alloc(0))
            }
        }

        let state, channel
        try {
            ;[state, channel] = await Promise.all([
                node.paymentChannels.contract.methods.states(pubKeyToEthereumAddress(counterparty)).call(
                    {
                        from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
                    },
                    'latest'
                ),
                node.paymentChannels.contract.methods.channels(channelId).call(
                    {
                        from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
                    },
                    'latest'
                )
            ])
        } catch (err) {
            log(node.peerInfo.id, err.message)
            return cb(null, Buffer.alloc(0))
        }

        if (!state.isSet) {
            log(node.peerInfo.id, Error(`Rejecting payment channel opening request because counterparty hasn't staked any funds.`))
            return cb(null, Buffer.alloc(0))
        }

        const stakedEther = new BN(state.stakedEther)
        const claimedFunds = new BN(restoreTx.value)

        // Check whether the counterparty has staked enough money to open
        // the payment channel
        if (stakedEther.lt(claimedFunds)) {
            log(
                node.peerInfo.id,
                `Rejecting payment channel opening request due to ${fromWei(claimedFunds.sub(stakedEther), 'ether')} ETH too less staked funds.`
            )
            return cb(null, Buffer.alloc(0))
        }
        // Check whether there is already such a channel registered in the
        // smart contract
        state = parseInt(channel.state)
        if (!Number.isInteger(state) || state < 0) {
            log(node.peerInfo.id, `Invalid state. Got ${state.toString()}.`)
            return cb(null, Buffer.alloc(0))
        }

        await node.db.put(node.paymentChannels.StashedRestoreTransaction(channelId), restoreTx.toBuffer(), { sync: true })

        node.paymentChannels.registerOpeningListener(channelId)
        node.paymentChannels.registerSettlementListener(channelId)

        const sigRestore = sign(restoreTx.hash, node.peerInfo.id.privKey.marshal())

        return cb(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
    }

    node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) =>
        pull(
            conn,
            lp.decode({
                maxLength: Transaction.SIZE
            }),
            pull.asyncMap(handleOpeningRequest),
            lp.encode(),
            conn
        )
    )
}
