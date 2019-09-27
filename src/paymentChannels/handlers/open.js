'use strict'

const Transaction = require('../../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { sign } = require('secp256k1')
const { pubKeyToEthereumAddress, numberToBuffer, bufferToNumber, log, getId, pubKeyToPeerId } = require('../../utils')
const paramap = require('pull-paramap')

const { fromWei } = require('web3-utils')
const BN = require('bn.js')

const { SIGNATURE_LENGTH } = require('../../transaction')

const { ChannnelState } = require('../enums.json')

module.exports = node => {
    const handleExistingRecord = async (channelId) => {
        const networkState = await node.paymentChannels.contract.methods.channels(channelId).call({
            from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
        })

        switch (parseInt(networkState.state)) {
            default:
                throw Error(`Payment channel ${channelId.toString('hex')} is already open. Cannot open it twice. Got '${state.state}'`)
            case ChannnelState.UNINITIALIZED:
                break
        }
    }
    /**
     * Checks whether the opening request is valid and whether it is plausible, i. e. the counterparty has enough
     * funds staked in the smart contract.
     *
     * @param {Buffer} channelId ID of the channel
     * @param {Buffer} counterparty compressed public key of the counterparty
     * @param {Transaction} restoreTx the backup transaction
     */
    const checkRequest = async (channelId, counterparty, restoreTx) => {
        let recordExists = false
        try {
            const state = await node.paymentChannels.state(channelId)

            recordExists = true

        } catch (err) {
            if (!err.notFound) throw err
        }

        if (recordExists) handleExistingRecord(channelId)

        if (bufferToNumber(restoreTx.index) !== 1) throw Error('Invalid payment channel opening request.')

        const [state, channel] = await Promise.all([
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

        if (!state.isSet)
            throw Error(
                `Rejecting payment channel opening request because counterparty ${(await pubKeyToPeerId(
                    counterparty
                )).toB58String()} hasn't staked any funds yet.`
            )

        const stakedEther = new BN(state.stakedEther)
        const claimedFunds = new BN(restoreTx.value)

        // Check whether the counterparty has staked enough money to open
        // the payment channel
        if (stakedEther.lt(claimedFunds))
            throw Error(`Rejecting payment channel opening request due to ${fromWei(claimedFunds.sub(stakedEther), 'ether')} ETH too less staked funds.`)

        // Check whether there is already such a channel registered in the
        // smart contract
        const channelState = parseInt(channel.state)
        if (!Number.isInteger(channelState) || channelState < 0) throw Error(`Invalid state. Got '${channelState.toString()}'.`)
    }

    const handleOpeningRequest = async (encodedTransaction, cb) => {
        if (encodedTransaction.length !== Transaction.SIZE) {
            log(node.peerInfo.id, 'Invalid size of payment channel opening request.')
            return cb(null, Buffer.alloc(0))
        }

        const restoreTransaction = Transaction.fromBuffer(encodedTransaction)

        const counterparty = restoreTransaction.counterparty

        const channelId = getId(
            /* prettier-ignore */
            pubKeyToEthereumAddress(counterparty),
            pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
        )

        try {
            await checkRequest(channelId, counterparty, restoreTransaction)
        } catch (err) {
            console.log(`${err.message}. Dropping request.`)
            return cb(null, Buffer.alloc(0))
        }

        node.paymentChannels.setState(channelId, {
            state: node.paymentChannels.TransactionRecordState.INITIALIZED,
            restoreTransaction,
            counterparty,
            nonce: restoreTransaction.nonce,
            preOpened: false
        })

        node.paymentChannels.registerOpeningListener(channelId)
        node.paymentChannels.registerSettlementListener(channelId)

        const sigRestore = sign(restoreTransaction.hash, node.peerInfo.id.privKey.marshal())

        return cb(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
    }

    node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) =>
        pull(
            conn,
            lp.decode({
                maxLength: Transaction.SIZE
            }),
            paramap(handleOpeningRequest, null, false),
            lp.encode(),
            conn
        )
    )
}
