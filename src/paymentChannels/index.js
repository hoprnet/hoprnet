'use strict'

const fs = require('fs')


const { bytesToHex } = require('web3').utils
const { recover } = require('secp256k1')

const { isPartyA, pubKeyToEthereumAddress, bufferToNumber, hash } = require('../utils')

const open = require('./open')
const close = require('./close')
const transfer = require('./transfer')
const settle = require('./settle')
const payout = require('./payout')

class PaymentChannel {
    constructor(node, contract) {
        this.openPaymentChannels = new Map()
        this.contract = contract

        this.node = node
        this.open = open(this)
        this.close = close(this)
        this.transfer = transfer(this)
        this.settle = settle(this)
        this.payout = payout(this)
    }

    setSettlementListener(channelId, listener = this.close) {
        let record
        if (this.has(channelId)) {
            record = this.openPaymentChannels.get(channelId.toString('base64'))
        } else {
            record = {}
        }

        record.listener = listener

        this.openPaymentChannels.set(channelId.toString('base64'), record)

        console.log('[\'' + this.node.peerInfo.id.toB58String() + '\']: Listening to channel ' + channelId.toString('hex'))
        this.contract.once('SettledChannel', {
            topics: [bytesToHex(channelId)]
        }, record.listener)
    }

    getEmbeddedMoney(from, tx) {
        const self = pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        const otherParty = pubKeyToEthereumAddress(from.pubKey.marshal())

        const last = this.get(tx.channelId)

        if (isPartyA(self, otherParty)) {
            return tx.value - last.value
        } else {
            return last.value - tx.value
        }
    }

    getCounterParty(channelId) {
        if (!this.has(channelId))
            return null

        const restoreTx = this.getRestoreTransaction(channelId)

        return recover(restoreTx.hash, restoreTx.signature, bufferToNumber(restoreTx.recovery))
    }

    set(tx) {
        let record
        if (this.has(tx.channelId)) {
            record = this.openPaymentChannels.get(tx.channelId.toString('base64'))
        } else {
            record = {}
        }

        record.tx = tx

        this.openPaymentChannels.set(tx.channelId.toString('base64'), record)
    }

    setRestoreTransaction(restoreTx) {
        let record
        if (this.has(restoreTx.channelId)) {
            record = this.openPaymentChannels.get(restoreTx.channelId.toString('base64'))
        } else {
            record = {}
        }

        record.restoreTx = restoreTx

        this.openPaymentChannels.set(restoreTx.channelId.toString('base64'), record)
    }

    get(channelId) {
        if (!this.has(channelId))
            return null

        return this.openPaymentChannels.get(channelId.toString('base64')).tx
    }

    getRestoreTransaction(channelId) {
        if (!this.has(channelId))
            return null

        return this.openPaymentChannels.get(channelId.toString('base64')).restoreTx
    }

    has(channelId) {
        return this.openPaymentChannels.has(channelId.toString('base64'))
    }

    delete(channelId) {
        this.openPaymentChannels.delete(channelId.toString('base64'))
    }

    import() {

    }

    export() {

    }
}

module.exports = PaymentChannel