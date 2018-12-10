'use strict'

const fs = require('fs')

const { bytesToHex } = require('web3').utils
const { isPartyA, getId, pubKeyToEthereumAddress } = require('../utils')

const open = require('./open')
const close = require('./close')
const transfer = require('./transfer')

class PaymentChannel {
    constructor(node, contract) {

        this.openPaymentChannels = new Map()

        this.contract = contract

        this.node = node
        this.open = open(this)
        this.close = close(this)
        this.transfer = transfer(this)
    }

    registerSettlementListener(channelId) {
        this.contract.once('SettledChannel', {
            filter: {
                channelId: bytesToHex(channelId)
            }
        }, this.close)
    }

    getEmbeddedMoney(from, tx) {
        const self = pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        const otherParty = pubKeyToEthereumAddress(from.pubKey.marshal())

        const lastValue = this.get(tx.channelId)

        if (isPartyA(self, otherParty)) {
            return tx.value - lastValue
        } else {
            return lastValue - tx.value
        }
    }

    set(tx) {
        this.openPaymentChannels.set(tx.channelId.toString('base64'), tx)
    }

    get(channelId) {
        return this.openPaymentChannels.get(channelId.toString('base64'))
    }

    has(channelId) {
        return this.openPaymentChannels.has(channelId.toString('base64'))
    }

    delete(channelId) {
        return this.openPaymentChannels.delete(channelId.toString('base64'))
    }

    import() {

    }

    export() {

    }
}

module.exports = PaymentChannel