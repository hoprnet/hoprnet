'use strict'

const withIs = require('class-is')
const crypto = require('crypto')
const defaultsDeep = require('@nodeutils/defaults-deep')
const forEachRight = require('lodash.foreachright')

const registerHandlers = require('./handlers')
const Header = require('./header')
const prp = require('./prp')
const Payment = require('../payments/index')

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
const Multihash = require('multihashes')
const bs58 = require('bs58')
const waterfall = require('async/waterfall')
const parallel = require('async/parallel')
const times = require('async/times')
const filter = require('async/filter')

const defer = require('pull-defer')
const Connection = require('interface-connection').Connection
const isFunction = require('lodash.isfunction')


const PACKET_SIZE = 500
const MAX_HOPS = 2

const PROTOCOL_VERSION = '0.0.1'
const PROTOCOL_NAME = 'ipfs'
const PROTOCOL_STRING = '/'.concat(PROTOCOL_NAME.toLowerCase()).concat('/').concat(PROTOCOL_VERSION)
const PROTOCOL_ACKNOWLEDGEMENT = '/'.concat(PROTOCOL_NAME.toLowerCase()).concat('/acknowledgement/').concat(PROTOCOL_VERSION)

const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')
const PADDING = Buffer.from('PADDING')
const PADDING_LENGTH = PADDING.length


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
    }

    start(output, cb) {
        waterfall([
            (cb) => super.start(err => {
                cb(err)
            }),
            (cb) => {
                this.handle(PROTOCOL_STRING, (protocol, conn) => {
                    pull(
                        conn,
                        pull.filter(data =>
                            data.length > 0 && Header.SIZE(MAX_HOPS) + PACKET_SIZE + PADDING_LENGTH === data.length
                        ),
                        pull.drain(data => {
                            const header = Header.fromBuffer(data.slice(0, Header.SIZE(MAX_HOPS)), MAX_HOPS)
                            header.forwardTransform(this.peerInfo.id.privKey.marshal())

                            const { key, iv } = Header.deriveCipherParameters(header.derivedSecret)
                            prp.createPRP(key, iv).inverse(data.slice(Header.SIZE(MAX_HOPS)))

                            const targetPeerId = PeerId.createFromBytes(header.address)
                            console.log('Node ' + this.peerInfo.id.toB58String() + ' sending to node ' + targetPeerId.toB58String() + '.')

                            parallel([
                                (cb) => {
                                    if (this.peerInfo.id.toBytes().compare(targetPeerId.toBytes()) === 0) {
                                        
                                        output(Hopper.removePadding(data.slice(Header.SIZE(MAX_HOPS))).toString())
                                    } else {
                                        waterfall([
                                            (cb) => this.peerRouting.findPeer(targetPeerId, cb),
                                            (peerInfo, cb) => this.dialProtocol(peerInfo, PROTOCOL_STRING, cb),
                                            (conn, cb) => {
                                                pull(
                                                    pull.once(data),
                                                    conn
                                                )
                                                cb(null)
                                            }
                                        ], cb)
                                    }
                                },
                                // (cb) => pull(
                                //     pull.once(Payments.createChallenge()),
                                //     conn
                                // )
                            ], (err) => {
                                output(err, null)
                            })
                        })
                    )


                })

                this.handle(PROTOCOL_ACKNOWLEDGEMENT, (protoocl, conn) => {
                    pull(
                        conn,
                        pull.drain(data => {
                            Payments.verifyChallenge()
                        })
                    )
                })

                cb(null, this)
            }
        ], cb)
    }

    static createNode(cb, output, options = {}) {
        function createAddress(id) {
            return Multiaddr(
                '/'.concat(PROTOCOL_NAME).concat('/').concat(id)
            )
        }

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
                    addr.encapsulate(createAddress(peerInfo.id.toB58String()))
                    peerInfo.multiaddrs.add(addr)
                })

                let node = new Hopper({
                    peerInfo
                })
                node.start(output, cb)
            }
        ], cb)
    }

    static addPadding(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a Buffer. Got \"' + typeof buf + '\" instead.')

        const bufLength = buf.length
        return Buffer.concat(
            [buf, PADDING, Buffer.alloc(PACKET_SIZE - ((bufLength + PADDING_LENGTH) % PACKET_SIZE)).fill(0)],
            Math.ceil(bufLength / PACKET_SIZE) * PACKET_SIZE + PADDING_LENGTH
        )
    }

    static removePadding(buf) {
        if (!Buffer.isBuffer(buf))
            throw Error('Expected a Buffer. Got \"' + typeof buf + '\" instead.')

        let lastIndex = buf.lastIndexOf(PADDING)
        if (lastIndex < 0)
            throw Error('String does not contain a valid padding.')

        return buf.slice(0, lastIndex)
    }

    sendMessage(msg, destination) {
        if (msg.length <= 0)
            throw Error('Expecting non-empty message.')

        let data = Buffer.isBuffer(msg) ? msg : Buffer.from(msg)

        data = Hopper.addPadding(data)

        times(data.length / (PACKET_SIZE + PADDING_LENGTH), (n, cb) => {
            this.sampleNodes(destination, (err, intermediateNodes) => {
                
                const { header, secrets, identifier } = Header.createHeader(intermediateNodes.concat([destination]), {
                    maxHops: MAX_HOPS
                })

                // Encrypt message
                forEachRight(secrets, secret => {
                    const { key, iv } = Header.deriveCipherParameters(secret)

                    prp.createPRP(key, iv).permutate(data)
                })

                this.dialProtocol(intermediateNodes[0], PROTOCOL_STRING, (err, conn) => {
                    if (err) { cb(err) }

                    pull(
                        pull.once(Buffer.concat([header.toBuffer(), data], Header.SIZE(MAX_HOPS) + PACKET_SIZE + PADDING_LENGTH)),
                        conn
                    )

                    cb(null, identifier)
                })
            })
        }, (err, identifiers) => {
            // console.log(err, identifiers)
        })
    }

    sampleNodes(destination, cb) {
        filter(this.peerBook.getAll(), (peerInfo, cb) => {
            const res =
                this.peerInfo.id.id.compare(peerInfo.id.id) !== 0 &&
                this.peerInfo.id.id.compare(destination.id) !== 0
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