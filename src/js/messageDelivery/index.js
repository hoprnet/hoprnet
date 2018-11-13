'use strict'

const withIs = require('class-is')
const defaultsDeep = require('@nodeutils/defaults-deep')

const Packet = require('./packet')
const registerHandlers = require('./handlers')
const c = require('./constants')
const crawlNetwork = require('./crawlNetwork')
const getPubKey = require('./getPubKey')

// DEMO
const { randomBytes } = require('crypto')
const { bufferToNumber, randomSubset } = require('../utils')
// END DEMO

const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const MUXER = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
const SECIO = require('libp2p-secio')
const libp2pCrypto = require('libp2p-crypto')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const wrtc = require('wrtc')
const WStar = require('libp2p-webrtc-star')
const WebRTC = new WStar({
    wrtc: wrtc
})

const pull = require('pull-stream')
const Multiaddr = require('multiaddr')
const bs58 = require('bs58')
const waterfall = require('async/waterfall')
const parallel = require('async/parallel')
const times = require('async/times')



const PACKET_SIZE = 500


// const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')



const ACKNOWLEDGEMENT_SIZE = 1000000

class Hopper extends libp2p {
    constructor(_options) {
        const defaults = {
            modules: {
                transport: [TCP],
                streamMuxer: [MUXER],
                connEncryption: [SECIO],
                dht: KadDHT,
                // peerDiscovery: [WebRTC.discovery]
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

        this.seenTags = new Set()
        this.pendingTransactions = new Map()
        this.crawlNetwork = crawlNetwork(this)
        this.getPubKey = getPubKey(this)
    }

    start(output, callback) {
        waterfall([
            (cb) => super.start(cb),
            // TODO: Fix libp2p's switch implementation / specify meaning of first parameter
            (_, cb) => registerHandlers(this, output, cb),
            // DEMO
            (node, cb) => {
                this.on('peer:connect', peer => {
                    console.log('[\'' + node.peerInfo.id.toB58String() + '\']: Incoming connection from \'' + peer.id.toB58String() + '\'.')
                    if (c.DEMO && this.peerBook.getAllArray().length == c.MAX_HOPS && process.argv.length == 2) {
                        setInterval(demo.bind(this), 4000)
                    }
                })
                cb(null, node)
            }
        ], callback)
    }

    static createNode(cb, output, options = {}) {
        options = defaultsDeep(options, {
            addrs: [],
            signallingServer: null // BOOTSTRAP_NODE
        })

        waterfall([
            (cb) => libp2pCrypto.keys.generateKeyPair('secp256k1', 256, cb),
            (key, cb) => key.public.hash((err, id) => cb(err, key, id)),
            (key, id, cb) => PeerInfo.create(new PeerId(id, key, key.public), cb),
            (peerInfo, cb) => {
                // TCP
                options.addrs.push(Multiaddr('/ip4/0.0.0.0/tcp/0'))

                // WebRTC
                if (options.signallingServer) {
                    options.addrs.push(
                        options.signallingServer
                            .encapsulate(multiaddr('/ws/p2p-webrtc-star/'))
                    )
                }

                options.addrs.forEach(addr => {
                    addr.encapsulate('/'.concat(c.PROTOCOL_NAME).concat('/').concat(peerInfo.id.toB58String()))
                    peerInfo.multiaddrs.add(addr)
                })

                let node = new Hopper({
                    peerInfo
                })
                node.start(output, cb)
            }
        ], cb)
    }

    sendMessage(msg, destination, cb) {
        if (!msg)
            throw Error('Expecting non-empty message.')

        switch (typeof msg) {
            case 'number':
                msg = msg.toString()
            case 'string':
                msg = Buffer.from(msg)
                break
            default:
                throw Error('Invalid input value. Got \"' + typeof msg + '\".')
        }

        times(Math.ceil(msg.length / c.PACKET_SIZE), (n, cb) => waterfall([
            (cb) => this.getIntermediateNodes(destination.id, cb),
            (intermediateNodes, cb) =>
                Packet.createPacket(
                    this,
                    msg.slice(n * c.PACKET_SIZE, Math.min(msg.length, (n + 1) * c.PACKET_SIZE)),
                    intermediateNodes,
                    destination,
                    (err, packet) => cb(err, packet, intermediateNodes[0])
                ),
            (packet, firstNode, cb) => this.dialProtocol(firstNode, c.PROTOCOL_STRING, (err, conn) => cb(err, conn, packet)),
            (conn, packet, cb) => {
                pull(
                    pull.once(packet.toBuffer()),
                    conn
                )
                cb()
            }
        ], cb), cb)
    }

    getIntermediateNodes(destination, cb) {
        const comparator = (peerInfo) => {
            return this.peerInfo.id.id.compare(peerInfo.id.id) !== 0 &&
                destination.id.compare(peerInfo.id.id) !== 0
        }
        waterfall([
            (cb) => this.crawlNetwork(cb, comparator),
            (cb) => cb(null, randomSubset(
                this.peerBook.getAllArray(), c.MAX_HOPS - 1, comparator))
        ], cb)
    }
}

function demo() {
    this.sendMessage('HelloWorld ' + Date.now().toString(), this.peerBook.getAllArray()[bufferToNumber(randomBytes(4)) % (this.peerBook.getAllArray().length)])
}

module.exports = withIs(Hopper, { className: 'hopper', symbolName: '@validitylabs/hopper/hopper' })