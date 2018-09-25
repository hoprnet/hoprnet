'use strict'

const constants = require('../constants')
const Web3 = require('web3')
const utils = require('../utils')
const moment = require('moment')

const waterfall = require('async/waterfall')
const parallel = require('async/parallel')

const pull = require('pull-stream')
const pullJson = require('pull-json-doubleline')

const web3 = new Web3(new Web3.providers.HttpProvider('http://localhost:8545'))

class PaymentChannel {
    constructor(self, pubKey, startTx) {
        this.self = self
        this.otherPubKey = pubKey
        this.currentState = startTx

        this.timelock = startTx.body.timelock
        this.closed = false

        this.timout = setTimeout(PaymentChannel.close, this.timelock - moment().valueOf(), this.currentState)
    }

    // dummy function
    isValid() {
        if (this.timelock.isSameOrAfter(moment())) {
            return false
        } else {
            return true
        }
        // this.contract.isValid(...).call(...)
    }

    // dummy function
    renew(cb) {
        if (this.closed)
            throw Error('Trying to renew an already closed payment channel.')

        this.timout.refresh()
        this.timelock = moment().add(2, 'hours').valueOf()

        cb()
    }

    static close(tx) {
        if (this.closed)
            throw Error('Trying to close an already closed payment channel.')
        
        if (moment().isBefore(tx.body.timelock))
            throw Error('Transaction is not yet valid.')

        this.closed = true
    }

    // dummy function
    static initiate(self, recipient, amount, cb) {
        let tx = {
            body: {
                amount: amount,
                timelock: moment().add(2, 'hours').valueOf()
            }
        }

        waterfall([
            (cb) => self.peerInfo.id.privKey.sign(PaymentChannel.toSignable(tx), (err, signature) => {
                tx.signatureA = signature
                cb(null)
            }),
            (cb) => self.peerRouting.findPeer(recipient, cb),
            (recipient, cb) => self.dialProtocol(recipient, constants.paymentChannelProtocol, cb),
            (conn, cb) => pull(
                pull.once(tx),
                pullJson.stringify(),
                conn,
                pull.map(str => utils.parseJSON(str)),
                pull.collect((err, data) => {
                    if (err) { throw err }

                    cb(null, data[0])
                })
            ),
            (tx, cb) => {
                recipient.pubKey.verify(PaymentChannel.toSignable(tx), tx.signatureB, cb)
            }
        ],
            (err, result) => {
                if (err) { throw err }
                if (!result)
                    cb(Error('General error.'), null)

                cb(null, new PaymentChannel(self, recipient, tx))
            }
        )
    }

    static establish(self, otherParty, tx, cb) {
        let signable = PaymentChannel.toSignable(tx)
        waterfall([
            (cb) => otherParty.id.pubKey.verify(signable, tx.signatureA, (err, valid) => {
                if (err) {
                    cb(err)
                } else if (!valid) {
                    cb(Error('General error'))
                } else {
                    cb(null)
                }
            }),
            (cb) => self.peerInfo.id.privKey.sign(signable, cb)
        ], (err, signatureB) => {
            if (err) {
                cb(err)
            } else {
                tx.signatureB = signatureB

                cb(null, tx, new PaymentChannel(self, otherParty, tx))
            }
        }
        )
    }

    // dummy function
    update(transaction, cb) {
        if (this.closed)
            throw Error('Trying to update an already closed payment channel.')

        this.verifyUpdateTransaction(transaction, (err, valid) => {
            if (err) { throw err }
            if (!valid)
                throw Error('General error.')

            this.currentState = transaction

            cb(null, this)
        })
    }

    verifyUpdateTransaction(tx, cb) {
        if (!tx || !tx.hasOwnProperty('body') || !tx.hasOwnProperty('signatureA') || !tx.hasOwnProperty('signatureB'))
            throw Error('Invalid transaction format.')

        if (!tx.body.hasOwnProperty('amount') || !tx.body.hasOwnProperty('timelock'))
            throw Error('Invalid transaction body')

        if (tx.amount > this.currentState.body.amount)
            throw Error('Insufficient funds.')

        let toVerify = PaymentChannel.toSignable(tx)

        parallel([
            (cb) => this.self.peerInfo.id.pubKey.verify(toVerify, tx.signatureA, cb),
            (cb) => this.self.peerInfo.id.pubKey.verify(toVerify, tx.signatureB, cb),
            (cb) => this.otherPubKey.pubKey.verify(toVerify, tx.signatureA, cb),
            (cb) => this.otherPubKey.pubKey.verify(toVerify, tx.signatureB, cb)
        ], (err, valid) => {
            if (err) { throw err }
            cb(null, valid[0] && valid[3] || valid[1] && valid[2])
        })
    }

    static toSignable(tx) {
        return utils.hash(
            Buffer.from(
                JSON.stringify(tx.body)
            )
        )
    }
}

module.exports = PaymentChannel