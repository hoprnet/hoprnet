'use strict'

const defaultsDeep = require('@nodeutils/defaults-deep')
const { waterfall, parallel } = require('neo-async')
const { generateKeyPair } = require('libp2p-crypto').keys
const { deserializeKeyPair, serializeKeyPair } = require('./utils')
const { randomBytes } = require('crypto')
const chacha = require('chacha')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const c = require('./constants')
const scrypt = require('scrypt')

const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')

module.exports = (options, db, cb) => {
    if (typeof db === 'function') {
        cb = db
        db = options
        options = {}
    }

    options = defaultsDeep(options, {
        addrs: [],
        signallingServer: null // BOOTSTRAP_NODE
    })

    options.addrs = options.addrs.map(addr => Multiaddr(addr))

    waterfall([
        (cb) => db.get('key-pair', (err, serializedKeyPair) => {
            if (err && !err.notFound) {
                throw err
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
        }),
        (peerId, cb) => PeerInfo.create(peerId, cb),
        (peerInfo, cb) => {
            // TCP
            options.addrs.push(Multiaddr('/ip4/0.0.0.0/tcp/0'))



            // WebRTC
            if (options.signallingServer) {
                options.addrs.push(
                    options.signallingServer
                        .encapsulate(Multiaddr('/ws/p2p-webrtc-star/'))
                )
            }

            options.addrs.forEach(addr => {
                addr.encapsulate('/'.concat(c.PROTOCOL_NAME).concat('/').concat(peerInfo.id.toB58String()))
                peerInfo.multiaddrs.add(addr)
            })

            cb(null, peerInfo)
        }
    ], cb)
}