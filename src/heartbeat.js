'use strict'

const { PROTOCOL_HEARTBEAT } = require('./constants')
const lp = require('pull-length-prefixed')
const { randomBytes, createHash } = require('crypto')
const { log } = require('./utils')
const { waterfall, each } = require('neo-async')
const pull = require('pull-stream')

const THIRTY_ONE_SECONDS = 31 * 1000

module.exports = (node) =>
    setInterval(() => {
        each(node.peerBook.getAllArray(), (peer, cb) => {
            if (Date.now() - node.peerBook.getAll()[peer.id.toB58String()].lastSeen > THIRTY_ONE_SECONDS) {
                console.log(node.peerBook.getAll()[peer.id.toB58String()].lastSeen)
                return waterfall([
                    (cb) => node.peerRouting.findPeer(peer.id, cb),
                    (peerInfo, cb) => node.dialProtocol(peerInfo, PROTOCOL_HEARTBEAT, cb),
                    (conn, cb) => {
                        const challenge = randomBytes(16)
        
                        pull(
                            pull.once(challenge),
                            lp.encode(),
                            conn,
                            lp.decode(),
                            pull.collect((err, hashValues) => {
                                if (err)
                                    return cb(err)
                                    
                                const response = createHash('sha256').update(challenge).digest().slice(0, 16)
                                node.peerBook.getAll()[peer.id.toB58String()].lastSeen = Date.now()
    
                                if (hashValues.length != 1 || hashValues[0].compare(response) !== 0)
                                    return cb(Error(`Invalid response. Got ${typeof hashValues} instead of ${response.toString('hex')}`))
                            })
                        )
                    }
                ], (err) => {
                    if (err) {
                        log(node.peerInfo.id, `Removing ${peer.id.toB58String()} from peerBook due to "${err.message}".`)
                        node.peerBook.remove(peer)
                    }
                    cb()
                })
            } else {
                return cb()
            }
        })
    }, THIRTY_ONE_SECONDS)
