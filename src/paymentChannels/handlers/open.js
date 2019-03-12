'use strict'

const Transaction = require('../../transaction')

const { PROTOCOL_PAYMENT_CHANNEL } = require('../../constants')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { sign } = require('secp256k1')
const { deepCopy, pubKeyToEthereumAddress, numberToBuffer, bufferToNumber, log } = require('../../utils')
const { applyEachSeries, applyEach } = require('neo-async')
const { fromWei } = require('web3-utils')
const BN = require('bn.js')
const Record = require('../record')

const { SIGNATURE_LENGTH } = require('../../transaction')

module.exports = (node) => node.handle(PROTOCOL_PAYMENT_CHANNEL, (protocol, conn) => pull(
    conn,
    lp.decode(),
    pull.asyncMap((data, cb) => {
        if (data.length !== Transaction.SIZE)
            return cb()

        const restoreTx = Transaction.fromBuffer(data)

        if (bufferToNumber(restoreTx.index) !== 1)
            return cb()

        const counterparty = restoreTx.counterparty

        applyEachSeries([
            (cb) => applyEach([
                // Check whether the counterparty has staked enough money to open
                // the payment channel
                (cb) => node.paymentChannels.contract.methods.states(pubKeyToEthereumAddress(counterparty)).call({
                    from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
                }, 'latest', (err, state) => {
                    if (err)
                        return cb(err)

                    if (!state.isSet)
                        return cb(Error(`Rejecting payment channel opening request because counterparty hasn't staked any funds.`))

                    const stakedEther = new BN(state.stakedEther)
                    const claimedFunds = new BN(restoreTx.value)
                    if (stakedEther.lt(claimedFunds))
                        return cb(Error(`Rejecting payment channel opening request due to ${fromWei(claimedFunds.sub(stakedEther), 'ether')} ETH too less staked funds.`))

                    return cb()
                }),
                // Check whether there is already such a channel registered in the
                // smart contract
                (cb) => {
                    const channelId = restoreTx.getChannelId(node.peerInfo.id)
                    node.paymentChannels.contract.methods.channels(channelId).call({
                        from: pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal())
                    }, (err, channel) => {
                        if (err)
                            return cb(err)

                        const state = parseInt(channel.state)
                        if (!Number.isInteger(state) || state < 0)
                            return cb(Error(`Invalid state. Got ${state.toString()} instead.`))

                        return cb()
                    })
                }
            ], cb),
            // Save channel information
            (cb) => node.paymentChannels.setChannel(
                Record.create(
                    restoreTx,
                    deepCopy(restoreTx, Transaction),
                    restoreTx.index, 
                    restoreTx.value,
                    (new BN(restoreTx.value)).imuln(2).toBuffer('be', Transaction.VALUE_LENGTH)),
                { sync: true }, cb),
        ], (err) => {
            if (err) {
                log(node.peerInfo.id, err.message)
                return cb()
            }

            node.paymentChannels.registerSettlementListener(restoreTx.getChannelId(node.peerInfo.id))

            const sigRestore = sign(restoreTx.hash, node.peerInfo.id.privKey.marshal())
            return cb(null, Buffer.concat([sigRestore.signature, numberToBuffer(sigRestore.recovery, 1)], SIGNATURE_LENGTH + 1))
        })
    }),
    pull.filter(data => Buffer.isBuffer(data)),
    lp.encode(),
    conn
))