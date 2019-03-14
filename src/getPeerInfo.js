'use strict'
require('dotenv').config()
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

    const getFromDatabase = (cb) =>
        db.get('key-pair', (err, serializedKeyPair) => {
            if (err && !err.notFound)
                return cb(err)

            if (!err)
                return deserializeKeyPair(serializedKeyPair, cb)

            let key, peerId
            waterfall([
                (cb) => generateKeyPair('secp256k1', 256, cb),
                (_key, cb) => {
                    key = _key
                    key.public.hash(cb)
                },
                (id, cb) => {
                    peerId = new PeerId(id, key, key.public)

                    serializeKeyPair(peerId, cb)
                },
                (cb) => db.put('key-pair', serializedKeyPair, cb)
            ], (err) => {
                if (err)
                    throw err

                cb(null, peerId)
            })
        })

    waterfall([
        (cb) => {
            if (PeerId.isPeerId(options.peerId))
                return cb(null, options.peerId)

            if (options['bootstrap-node'])
                return privKeyToPeerId(process.env.FUND_ACCOUNT_PRIVATE_KEY, cb)

            getFromDatabase(cb)
        },
        (peerId, cb) => {
            const peerInfo = new PeerInfo(peerId)
            options.addrs.forEach((addr) => {
                // peerInfo.multiaddrs.add(addr.encapsulate(`/${PROTOCOL_NAME}/${peerInfo.id.toB58String()}`))
                peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${peerInfo.id.toB58String()}`))
                // peerInfo.multiaddrs.add(`/dns4/hopr.validity.io/tcp/9092/ws/p2p-webrtc-star/${PROTOCOL_NAME}/${peerInfo.id.toB58String()}`)

            })

            return cb(null, peerInfo)
        }
    ], cb)

}