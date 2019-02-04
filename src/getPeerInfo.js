'use strict'

const defaultsDeep = require('@nodeutils/defaults-deep')
const { waterfall } = require('neo-async')
const { generateKeyPair } = require('libp2p-crypto').keys
const { deserializeKeyPair, serializeKeyPair } = require('./utils')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multihash = require('multihashes')
const { createHash } = require('crypto')
const Multiaddr = require('multiaddr')
const c = require('./constants')

const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')

module.exports = (options, db, cb) => {
    if (typeof db === 'function') {
        if (!options.peerInfo) {
            return cb(Error('Invalid input parameter. Please set a valid peerInfo.'))
        }
        cb = db
    }

    if (PeerInfo.isPeerInfo(options.peerInfo)) {
        return cb(null, options.peerInfo)
    }

    if (typeof options.peerInfo === 'string') {
        options.peerInfo = new PeerId(Multihash.encode(createHash('sha256').update(options.peerInfo).digest(), 'sha2-256'))
    }

    if (options.addrs && Array.isArray(options.addrs)) {
        options.addrs = options.addrs.map(addr => Multiaddr(addr))
    }
    
    options = defaultsDeep(options, {
        addrs: [],
        signallingServer: null // BOOTSTRAP_NODE
    })
    
    const addAddrs = (peerInfo, cb) => {
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

        // Delete properties that has now become unnecessary
        delete options.addrs
        delete options.signallingServer

        cb(null, peerInfo)
    }

    const getFromDatabase = (cb) => db.get('key-pair', (err, serializedKeyPair) => {
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
    })

    return waterfall([
        (cb) => {
            if (PeerId.isPeerId(options.peerInfo)) {
                cb(null, options.peerInfo)
            } else {
                getFromDatabase(cb)
            }
        },
        (peerId, cb) => PeerInfo.create(peerId, cb),
        addAddrs
    ], cb)

}