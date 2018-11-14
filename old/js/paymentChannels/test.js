'use strict'

const testing = require('../testing/')
const waterfall = require('async/waterfall')
const parallel = require('async/parallel')

const PaymentChannel = require('./index')
const handler = require('./handlers')

const cloneDeep = require('lodash.clonedeep')

let amount = 3
let moneyAmount = 5

waterfall([
    (cb) => testing.createNodes(amount, null, cb),
    (nodes, cb) => {
        nodes.forEach(node => handler.registerFunctionality(node, null))
        cb(null, nodes)
    },
    (nodes, cb) => testing.warmUpNodes(nodes, (err) => cb(err, nodes)),
],
    (err, nodes) => {
        if (err) { throw err }

        PaymentChannel.initiate(nodes[0], nodes[1].peerInfo.id, moneyAmount, (err, channel) => {
            if (err) { throw err }

            let tx = cloneDeep(channel.currentState)

            tx.body.amount = tx.body.amount - 2

            let toSign = PaymentChannel.toSignable(tx)
            parallel([
                (cb) => nodes[0].peerInfo.id.privKey.sign(toSign, cb),
                (cb) => nodes[1].peerInfo.id.privKey.sign(toSign, cb)
            ], (err, signatures) => {
                if (err) { throw err }

                tx.signatureA = signatures[0]
                tx.signatureB = signatures[1]

                channel.update(tx, (err, channel) => {
                    console.log(channel)
                })
            })
        })
    }
)