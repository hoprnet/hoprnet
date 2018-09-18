'use strict'

const pull = require('pull-stream')
const PeerId = require('peer-id')
const Header = require('./header')
const prp = require('./prp')
const multihash = require('multihashes')

const payments = require('../payments/index')
const constants = require('../constants')

module.exports = (node, secretKey) => {
    node.handle(constants.relayProtocol, (err, conn) => {
        pull(
            conn,
            processMessage(secretKey, node, console.log),
            conn
        )
    })
}

function processMessage(secretKey, node, callback) {
    return function (read) {
        return function readable(end, reply) {
            let header = {}
            let state = 0
            read(end, function next(end, data) {
                if (!end) {
                    switch (state) {
                        case 0:
                            header.alpha = data
                            state++
                            read(end, next)
                            break
                        case 1:
                            header.beta = data
                            state++
                            read(end, next)
                            break
                        case 2:
                            header.gamma = data
                            header = new Header(header.alpha, header.beta, header.gamma).forwardTransform(secretKey)
                            state++
                            read(end, next)
                        case 3:
                            const { key, iv } = Header.deriveCipherParameters(header.derived_secret)
                            const plaintext = prp.createPRP(key, iv).inverse(data).toString()

                            const ownhash = multihash.decode(node.peerInfo.id.toBytes()).digest

                            if (ownhash.compare(header.address) === 0) {
                                callback(null, plaintext)
                            } else {
                                forwardMessage(node, header, plaintext)
                            }
                            reply(end, payments.createAckMessage(node.peerInfo.id.toBytes()))
                            break
                        default:
                            throw Error('Unable to parse header.')
                    }
                } else {
                    reply(end, null)
                }
            })
        }
    }
}

function forwardMessage(node, header, msg) {
    node.peerRouting.findPeer(multihash.encode(header.address, 'sha2-256'), (err, peerInfo) => {
        if (err) { throw err }

        node.dialProtocol(peerInfo, constants.relayProtocol, (err, conn) => {
            if (err) { throw err }

            pull(
                pull.values([header.alpha, header.beta, header.gamma, msg]),
                conn,
                pull.collect((err, ack) => {
                    if (err) { throw err }

                    payments.verifyAckMessage(ack)
                })
            )
        })
    })

}