'use strict'

const { PROTOCOL_HEARTBEAT } = require('../constants')
const lp = require('pull-length-prefixed')
const { randomBytes, createHash } = require('crypto')
const { log } = require('../utils')
const { waterfall, each } = require('neo-async')
const pull = require('pull-stream')

const THIRTY_ONE_SECONDS = 31 * 1000

module.exports = (node) => setInterval(() => 
    each(node.peerBook.getAllArray(), (peerInfo, cb) => {
        if (
            // node was in the list when last round of heartbeat started
            // make sure that it is still here
            !node.peerBook.getAll()[peerInfo.id.toB58String()] ||
            // check whether node was seen recently
            Date.now() - (node.peerBook.getAll()[peerInfo.id.toB58String()].lastSeen || 0) <= THIRTY_ONE_SECONDS
        )
            return cb()

        waterfall([
            (cb) => {
                if (!peerInfo.isConnected() || peerInfo.multiaddrs.size < 1)
                    return node.peerRouting.findPeer(peerInfo.id, cb)

                cb(null, peerInfo)
            },
            (peerInfo, cb) => {
                // console.log(`Heartbeat dialing ${peerInfo.multiaddrs.toArray().join(', ')}.`)

                node.dialProtocol(peerInfo, PROTOCOL_HEARTBEAT, cb)
            },
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
                        if (node.peerBook.has(peerInfo.id.toB58String())) {
                            node.peerBook.getAll()[peerInfo.id.toB58String()].lastSeen = Date.now()
                        } else {
                            log(node.peerInfo.id, `Heartbeat: Accidentially storing peerId ${peerInfo.id.toB58String()} in peerBook.`)
                            node.peerBook.put(peerInfo)
                        }

                        if (hashValues.length != 1 || hashValues[0].compare(response) !== 0)
                            return cb(Error(`Invalid response. Got ${typeof hashValues} instead of ${response.toString('hex')}`))

                        return cb()
                    })
                )
            }
        ], (err) => {
            if (err) {
                log(node.peerInfo.id, `Removing ${peerInfo.id.toB58String()} from peerBook due to "${err.message}".`)

                return node.hangUp(peerInfo, cb)

                // node._dht.routingTable.remove(peer.id, () => {

                //     node.peerBook.remove(peer)
                // })
            }

            return cb()
        })
    }), THIRTY_ONE_SECONDS)
