'use strict'

const EventEmitter = require('events');
const Transaction = require('../transaction')

const { BN } = require('web3').utils
const toPull = require('stream-to-pull-stream')
const pull = require('pull-stream')

const { isPartyA, pubKeyToEthereumAddress, sendTransaction, log } = require('../utils')

const open = require('./open')
const close = require('./close')
const transfer = require('./transfer')
const settle = require('./settle')
const payout = require('./payout')

const HASH_LENGTH = 32

class PaymentChannel extends EventEmitter {
    constructor(node, contract, nonce) {
        super()

        this.nonce = nonce
        this.contract = contract

        this.node = node
        this.open = open(this)
        this.close = close(this)
        this.transfer = transfer(this)
        this.settle = settle(this)
        this.payout = payout(this)
    }

    static createPaymentChannels(node, contract, cb) {
        node.web3.eth.getTransactionCount(pubKeyToEthereumAddress(node.peerInfo.id.pubKey.marshal()), (err, nonce) => {
            if (err) { throw err }

            node.paymentChannels = new PaymentChannel(node, contract, nonce)

            cb()
        })
    }

    setSettlementListener(channelId, listener = this.close) {
        if (!Buffer.isBuffer(channelId) || channelId.length !== HASH_LENGTH)
            throw Error(`Invalid input parameter. Expected a Buffer of size ${HASH_LENGTH} but got ${typeof channelId}.`)

        log(this.node.peerInfo.id, `Listening to channel \x1b[33m${channelId.toString('hex')}\x1b[0m`)

        this.contract.once('ClosedChannel', {
            topics: [`0x${channelId.toString('hex')}`]
        }, listener)
    }

    getEmbeddedMoney(channelId, receivedTx, cb) {
        this.getChannel(channelId, (err, { restoreTx, currentValue }) => {
            if (err)
                throw err

            currentValue = new BN(currentValue)
            const newValue = new BN(receivedTx.value)

            const self = pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
            const otherParty = pubKeyToEthereumAddress(restoreTx.counterparty)

            if (isPartyA(self, otherParty)) {
                cb(null, newValue.isub(currentValue))
            } else {
                cb(null, currentValue.isub(newValue))
            }
        })
    }

    setChannel(newRecord, channelId, cb) {
        if (typeof channelId === 'function') {
            if (!newRecord.restoreTx)
                throw Error('Unable to compute channelId.')

            cb = channelId
            channelId = newRecord.tx.getChannelId(this.node.peerInfo.id)
        }

        if (!channelId || !Buffer.isBuffer(channelId) || channelId.length !== 32)
            throw Error('Unable to determine channelId.')

        const key = this.getKey(channelId)

        this.getChannel(channelId, (err, record = {}) => {
            if (err)
                throw err

            Object.assign(record, newRecord)

            this.node.db.put(key, this.toBuffer(record), cb)
        })
    }

    getKey(channelId) {
        return Buffer.concat([Buffer.from('payments-channel-'), channelId], 17 + 32)
    }

    toBuffer(record) {
        return Buffer.concat([
            record.tx ? record.tx.toBuffer() : Buffer.alloc(Transaction.SIZE, 0),
            record.restoreTx ? record.restoreTx.toBuffer() : Buffer.alloc(Transaction.SIZE, 0),
            record.index ? record.index : Buffer.alloc(Transaction.INDEX_LENGTH, 0),
            record.currentValue ? record.currentValue : Buffer.alloc(Transaction.VALUE_LENGTH, 0),
            record.totalBalance ? record.totalBalance : Buffer.alloc(Transaction.VALUE_LENGTH, 0)
        ], Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH + Transaction.VALUE_LENGTH)
    }
    fromBuffer(buf) {
        return {
            tx: Transaction.fromBuffer(buf.slice(0, Transaction.SIZE)),
            restoreTx: Transaction.fromBuffer(buf.slice(Transaction.SIZE, Transaction.SIZE + Transaction.SIZE)),
            index: buf.slice(Transaction.SIZE + Transaction.SIZE, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH),
            currentValue: buf.slice(Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH),
            totalBalance: buf.slice(Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH, Transaction.SIZE + Transaction.SIZE + Transaction.INDEX_LENGTH + Transaction.VALUE_LENGTH + Transaction.VALUE_LENGTH)
        }
    }

    getChannel(channelId, cb) {
        const key = this.getKey(channelId)

        this.node.db.get(key, (err, record) => {
            if (err) {
                if (err.notFound) {
                    cb(null)
                } else {
                    cb(err)
                }
            } else {
                cb(null, this.fromBuffer(record))
            }
        })
    }

    deleteChannel(channelId, cb) {
        const key = `payments-channel-${channelId.toString('base64')}`

        this.node.db.del(key, cb)
    }

    getChannels() {
        return pull(
            toPull(this.node.db.createReadStream({
                // payments-channel-\000...\000
                gt: this.getKey(Buffer.alloc(32, 0)),
                // payments-channel-\255...\255
                lt: this.getKey(Buffer.alloc(46, 255))
            })),
            pull.map(record => Object.assign(record, {
                value: this.fromBuffer(record.value)
            }))
        )
    }

    /**
     * Takes a transaction object generetad by web3.js and publishes it in the
     * network. It automatically determines the necessary amount of gas i
     * 
     * @param {Object} txObject the txObject generated by web3.js
     * @param {Web3} web3 a web3.js instance
     * @param {Function} cb the function to be called afterwards
     */
    async contractCall(txObject, value, cb = () => { }) {
        if (typeof value === 'function') {
            cb = value
            value = '0'
        }

        const estimatedGas = await txObject.estimateGas({
            from: pubKeyToEthereumAddress(this.node.peerInfo.id.pubKey.marshal())
        })

        this.nonce = this.nonce + 1

        sendTransaction({
            to: this.contract._address,
            nonce: this.nonce - 1,
            gas: estimatedGas,
            data: txObject.encodeABI()
        }, this.node.peerInfo.id, this.node.web3, (err, receipt) => {
            if (err)
                throw err

            cb(null, receipt)
        })
    }
}

module.exports = PaymentChannel