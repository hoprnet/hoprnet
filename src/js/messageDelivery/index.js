'use strict'

const withIs = require('class-is')
const defaultsDeep = require('@nodeutils/defaults-deep')

const Packet = require('./packet')
const registerHandlers = require('./handlers')
const c = require('./constants')


const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const MUXER = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
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
const filter = require('async/filter')



const PACKET_SIZE = 500
const MAX_HOPS = 4



const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')



const ACKNOWLEDGEMENT_SIZE = 1000000

class Hopper extends libp2p {
    constructor(_options) {
        const defaults = {
            modules: {
                transport: [TCP],
                streamMuxer: [MUXER],
                // connEncryption: [SECIO],
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
    }

    start(output, callback) {
        waterfall([
            (cb) => super.start(err => {
                cb(err)
            }),
            (cb) => registerHandlers(this, output, cb)
        ], callback)
    }

    static createNode(cb, output, options = {}) {
        options = defaultsDeep(options, {
            addrs: [],
            signallingServer: null // BOOTSTRAP_NODE
        })

        waterfall([
            (cb) => libp2pCrypto.keys.generateKeyPair('secp256k1', 256, cb),
            (key, cb) => waterfall([
                (cb) => key.public.hash(cb),
                (id, cb) => PeerInfo.create(new PeerId(id, key, key.public), cb)
            ], cb),
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

    sendMessage(msg, destination) {
        if (!msg)
            throw Error('Expecting non-empty message.')

        switch(typeof msg) {
            case 'number':
                msg = msg.toString()
            case 'string':
                msg = Buffer.from(msg)
                break
            default:
                throw Error('Invalid input value. Got \"' + typeof msg + '\".')
        }

        times(Math.ceil(msg.length / c.PACKET_SIZE),
            (n, cb) => waterfall([
                (cb) => this.sampleNodes(destination, cb),
                (intermediateNodes, cb) =>
                    Packet.createPacket(
                        this,
                        msg.slice(n * c.PACKET_SIZE, Math.min(msg.length, (n + 1) * c.PACKET_SIZE)),
                        intermediateNodes,
                        destination,
                        (err, packet) => cb(err, packet, intermediateNodes)
                    ),
                (packet, intermediateNodes, cb) => this.dialProtocol(intermediateNodes[0], c.PROTOCOL_STRING, (err, conn) => cb(err, conn, packet)),
                (conn, packet, cb) => {
                    pull(
                        pull.once(packet.toBuffer()),
                        conn
                    )
                    cb()
                }
            ], cb),
            (err, identifiers) => {
                // console.log(err, identifiers)
            }
        )
        // this.sampleNodes(destination, (err, intermediateNodes) => {
        //     intermediateNodes.concat([destination]).forEach(node => console.log(node.toB58String()))

        //     const { header, secrets, identifier } = Header.createHeader(intermediateNodes.concat([destination]))

        //     // Encrypt message
        //     forEachRight(secrets, secret => {
        //         const { key, iv } = Header.deriveCipherParameters(secret)
        //         console.log('Encrypting with ' + bs58.encode(secret))

        //         prp.createPRP(key, iv).permutate(data)
        //     })

        //     this.dialProtocol(intermediateNodes[0], PROTOCOL_STRING, (err, conn) => {
        //         if (err) { cb(err) }



        //         cb(null, identifier)
        //     })
        // })
    }

sampleNodes(destination, cb) {
    filter(this.peerBook.getAll(), (peerInfo, cb) => {
        const res =
            this.peerInfo.id.id.compare(peerInfo.id.id) !== 0 &&
            destination.id.compare(peerInfo.id.id) !== 0
        cb(null, res)
    }, (err, peerInfos) => {
        cb(null, peerInfos.slice(0, MAX_HOPS - 1).map(peerInfo => peerInfo.id))
    })
}

handleHeader(header, ciphertext) {
    console.log(header, ciphertext)
}
}

module.exports = withIs(Hopper, { className: 'hopper', symbolName: '@validitylabs/hopper/hopper' })