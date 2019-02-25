'use strict'

const { waterfall } = require('neo-async')
const { generateKeyPair } = require('libp2p-crypto').keys
const { deserializeKeyPair, serializeKeyPair, privKeyToPeerId } = require('./utils')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const { NAME } = require('./constants')

module.exports = (options, db, cb) => {
    if (typeof db === 'function') {
        if (!options.peerInfo)
            return cb(Error('Invalid input parameter. Please set a valid peerInfo.'))

        cb = db
    }

    if (PeerInfo.isPeerInfo(options.peerInfo))
        return cb(null, options.peerInfo)

    if (!options.addrs || !Array.isArray(options.addrs))
        return cb(Error('Unable to start node without an address. Please provide at least one.'))

    options.addrs = options.addrs.map(addr => Multiaddr(addr))

    const getFromDatabase = (cb) => db.get('key-pair', (err, serializedKeyPair) => {
        if (err && !err.notFound) {
            cb(err)
        } else if (err && err.notFound) {
            generateKeyPair('secp256k1', 256, (err, key) => waterfall([
                (cb) => key.public.hash(cb),
                (id, cb) => {
                    const peerId = new PeerId(id, key, key.public)

                    serializeKeyPair(peerId, (err, serializedKeyPair) => {
                        if (err)
                            throw err

                        db.put('key-pair', serializedKeyPair, (err) => {
                            if (err)
                                throw err

                            cb(null, peerId)
                        })
                    })
                }
            ], cb))
        } else {
            deserializeKeyPair(serializedKeyPair, cb)
        }
    })

    return waterfall([
        (cb) => {
            if (PeerId.isPeerId(options.peerId))
                return cb(null, options.peerId)

            if (options['bootstrap-node'])
                return privKeyToPeerId(require('../config/.secrets.json').fundAccountPrivateKey, cb)

            getFromDatabase(cb)
        },
        (peerId, cb) => PeerInfo.create(peerId, cb),
        (peerInfo, cb) => {
            options.addrs.forEach((addr) => {
                // peerInfo.multiaddrs.add(addr.encapsulate(`/${PROTOCOL_NAME}/${peerInfo.id.toB58String()}`))
                peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`))
                // peerInfo.multiaddrs.add(`/dns4/hopr.validity.io/tcp/9092/ws/p2p-webrtc-star/${PROTOCOL_NAME}/${peerInfo.id.toB58String()}`)

            })

            return cb(null, peerInfo)
        }
    ], cb)

}