'use strict'

const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const WS = require('libp2p-websockets')
const Mplex = require('libp2p-mplex')
const SPDY = require('libp2p-spdy')
const SECIO = require('libp2p-secio')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const KadDHT = require('libp2p-kad-dht')
const defaultsDeep = require('@nodeutils/defaults-deep')
const waterfall = require('async/waterfall')
const parallel = require('async/parallel')

const secp256k1 = require('secp256k1')
const multihash = require('multihashes')

const Record = require('../record')
const constants = require('./constants')
const HeaderTest = require('./messageDelivery/test')
const Header = require('./messageDelivery/header')
const messageDeliveryHandler = require('./messageDelivery/handlers')

const prp = require('./messageDelivery/prp')
const dht = require('../dht')


// should implement
// -> interface transport
// -> interface peerRouting

const pull = require('pull-stream')

const fs = require('fs')

const crypto = require('libp2p-crypto')

class MyBundle extends libp2p {
    constructor(_options) {
        const defaults = {
            modules: {
                transport: [TCP, WS],
                streamMuxer: [Mplex],
                // connEncryption: [SECIO],
                dht: KadDHT
            },
            config: {
                dht: {
                    kBucketSize: 20
                },
                EXPERIMENTAL: {
                    dht: true
                }
             }
        }

        super(defaultsDeep(_options, defaults))
    }
}

function createNode(callback, keyPath, addrs) {
    let node

    waterfall([
        (cb) => fs.readFile(keyPath, cb),
        (jsonKey, cb) => {
            crypto.keys.generateKeyPair('secp256k1', 256, (err, key) => {
                let id = multihash.encode(key.public.bytes, 'sha2-256')
                PeerInfo.create(new PeerId(id, key, key.public), cb)
            })
        },
        (peerInfo, cb) => {
            addrs.forEach(addr => peerInfo.multiaddrs.add(addr))
            node = new MyBundle({
                peerInfo
            })
            node.start(cb)
        }
    ], (err) => callback(err, node))
}


parallel([
    (cb) => createNode(cb, './keys/alice.json', ['/ip4/0.0.0.0/tcp/0']),
    (cb) => createNode(cb, './keys/bob.json', ['/ip4/0.0.0.0/tcp/0'])
    // (cb) => createNode(cb, './keys/chris.json', ['/ip4/0.0.0.0/tcp/0'])
], (err, nodes) => {
    if (err) { throw err }

    let foo = HeaderTest.createTestHeader(2)
    let msg = Buffer.from(
        'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ' +
        'ut labore et dolore magna aliqua. Netus et malesuada fames ac turpis egestas integer eget ' +
        'aliquet. Commodo viverra maecenas accumsan lacus vel. Mauris a diam maecenas sed enim ut sem ' +
        'viverra. Habitasse platea dictumst vestibulum rhoncus est. Eget nullam non nisi est sit amet ' +
        'facilisis. In ante metus dictum at tempor. Auctor augue mauris augue neque gravida in. Ac ' +
        'auctor augue mauris augue neque gravida. Sit amet aliquam id diam. Sed turpis tincidunt id ' +
        'aliquet risus feugiat. Tristique nulla aliquet enim tortor at auctor urna nunc id. Nec dui ' +
        'nunc mattis enim. Congue eu consequat ac felis.')
    let ciphertext = msg

    nodes.forEach((node, index) => {
        messageDeliveryHandler(node, foo.keys[index].privKey)
        // dht.registerHandlers(node)
    })

    const node1 = nodes[0]
    const node2 = nodes[1]
    // const node3 = nodes[2]

    foo.secrets.forEach(secret => {
        let { key, iv } = Header.deriveCipherParameters(secret)
        ciphertext = prp.createPRP(key, iv).permutate(ciphertext)
    })

    node2.dialProtocol(node1.peerInfo, constants.relayProtocol, (err, conn) => {
        pull(
            pull.values([foo.header.alpha, foo.header.beta, foo.header.gamma, ciphertext]),
            conn
        )
    })
})