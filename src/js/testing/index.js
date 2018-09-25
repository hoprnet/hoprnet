'use strict'

const libp2pCrypto = require('libp2p-crypto')
const crypto = require('crypto')
const secp256k1 = require('secp256k1')

const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const WS = require('libp2p-websockets')
const MUXER = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')

const waterfall = require('async/waterfall')
const times = require('async/times')
const multihash = require('multihashes')

const defaultsDeep = require('@nodeutils/defaults-deep')

class TestBundle extends libp2p {
    constructor(_options) {
        const defaults = {
            modules: {
                transport: [TCP, WS],
                streamMuxer: [MUXER],
                // connEncryption: [SECIO],
                dht: KadDHT
            },
            config: {
                dht: {
                    kBucketSize: 20
                },
                EXPERIMENTAL: {
                    // dht must be enabled
                    dht: true
                }
            }
        }
        super(defaultsDeep(_options, defaults))
    }
}

module.exports.createNodes = function (amount, addrs, callback) {
    addrs = addrs || []
    times(amount, (n, next) => {
        let node
        let _addrs = addrs[n] || ['/ip4/0.0.0.0/tcp/0', '/ip4/127.0.0.1/tcp/0/ws']

        waterfall([
            (cb) => {
                libp2pCrypto.keys.generateKeyPair('secp256k1', 256, (err, key) => {
                    let hash = crypto.createHash('sha256').update(key.public.bytes).digest()
                    let id = multihash.encode(hash, 'sha2-256')
                    PeerInfo.create(new PeerId(id, key, key.public), cb)
                })
            },
            (peerInfo, cb) => {
                _addrs.forEach(addr => peerInfo.multiaddrs.add(addr))

                node = new TestBundle({
                    peerInfo
                })
                node.start(cb)
            }
        ], (err) => {
            next(err, node)
        })
    }, callback)
}


module.exports.warmUpNodes = function (nodes, cb) {
    if (nodes.length <= 1) {
        cb(null)
    } else {
        nodes[0].dial(nodes[1].peerInfo, (err, conn) => {
            if (err) { throw err }
            module.exports.warmUpNodes(nodes.slice(1), cb)
        })
    }
}

function generateKeyPair() {
    let privKey, publicKey

    do {
        privKey = crypto.randomBytes(Header.PRIVATE_KEY_LENGTH)
    } while (!secp256k1.privateKeyVerify(privKey))
    publicKey = secp256k1.publicKeyCreate(privKey)

    return { privKey, publicKey }
}

module.exports.generateKeyPairs = function (amount) {
    let result = []

    for (let i = 0; i < amount; i++) {
        let { privKey, publicKey } = generateKeyPair

        result.push({
            privKey: privKey,
            publicKey: publicKey
        })
    }

    return result
}

module.exports.fundWallets = function () { }