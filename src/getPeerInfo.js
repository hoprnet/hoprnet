'use strict'

const defaultsDeep = require('@nodeutils/defaults-deep')
const { waterfall } = require('neo-async')
const { generateKeyPair } = require('libp2p-crypto').keys
const { deserializeKeyPair } = require('./utils')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const Multiaddr = require('multiaddr')
const c = require('./constants')


const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')

module.exports = (options = {}, db, cb) => {
    options = defaultsDeep(options, {
        addrs: [],
        signallingServer: null // BOOTSTRAP_NODE
    })

    db.get('key-pair', (err, serializedKeyPair) => {
        if (err && !err.notFound) {
            throw err
        } else if (err && err.notFound) {
            generateKeyPair('secp256k1', 256, (err, key) => waterfall([
                (cb) => key.public.hash(cb),
                (id, cb) => 
                
                PeerInfo.create(new PeerId(id, key, key.public), cb),
                (peerInfo, cb) => {

                }
            ]))

            generateKeyPair('secp256k1', 256, cb)
        } else {
            cb(null, deserializeKeyPair(serializedKeyPair))
        }


    })

    waterfall([
        (cb) => generateKeyPair('secp256k1', 256, cb),
        (key, cb) => key.public.hash((err, id) => cb(err, key, id)),
        (key, id, cb) => PeerInfo.create(new PeerId(id, key, key.public), cb),
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