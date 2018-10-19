'use strict'

const libp2pCrypto = require('libp2p-crypto')
const crypto = require('crypto')
const secp256k1 = require('secp256k1')
const bs58 = require('bs58')

const stun = require('stun')
const { STUN_BINDING_REQUEST, STUN_ATTR_XOR_MAPPED_ADDRESS } = stun.constants
const dgram = require('dgram')
const getPort = require('get-port')



const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const WS = require('libp2p-websockets')
const MUXER = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
const PeerInfo = require('peer-info')
const PeerId = require('peer-id')
const wrtc = require('wrtc')
const WStar = require('libp2p-webrtc-star')
const WebRTC = new WStar({
    wrtc: wrtc
})

const waterfall = require('async/waterfall')
const parallel = require('async/parallel')
const times = require('async/times')
const each = require('async/each')
const whilst = require('async/whilst')
const map = require('async/map')



const Multihash = require('multihashes')
const Multiaddr = require('multiaddr')


const defaultsDeep = require('@nodeutils/defaults-deep')

const pull = require('pull-stream')

class TestBundle extends libp2p {
    constructor(_options) {
        const defaults = {
            modules: {
                transport: [TCP],
                streamMuxer: [MUXER],
                // connEncryption: [SECIO],
                dht: KadDHT,
                peerDiscovery: [WebRTC.discovery]
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

module.exports.createNodes = (amount, callback) => {
    let ports = new Set()

    whilst(() => ports.size < amount, (cb) => {
        getPort().then(port => {
            ports.add(port)
            cb()
        })
    }, (err) => {
        map(ports, (port, cb) => {
            let node
            waterfall([
                (cb) => this.createPeerInfo(port, cb),
                (peerInfo, publicAddress, cb) => {
                    // peerInfo.multiaddrs.forEach(addr => {
                    //     if (addr.toOptions().host === '0.0.0.0') {
                    //         peerInfo.multiaddrs.delete(addr)
                    //     }
                    // })
                    node = new TestBundle({
                        peerInfo
                    })
                    node.publicAddress = publicAddress
                    node.start((err) => cb(err, node))
                }
                // (node, publicAddress, cb) => {
                //     //node.peerInfo.multiaddrs.add(publicAddress)
                //     cb(null, node)
                // }
            ], cb)
        }, callback)
    })
}

module.exports.createPeerInfo = (port, cb) => {
    let server
    waterfall([
        (cb) => {
            server = dgram.createSocket('udp4')
            server.on('listening', cb)
            server.bind({
                port: port
            });
        },
        (cb) => {
            cb(null, stun.createServer(server))
        },
        (stunServer, cb) => parallel({
            publicAddress: (cb) => stunServer.once('bindingResponse', (stunMsg) => {
                stunServer.close()
                server.close(
                    cb(null, Multiaddr.fromNodeAddress(stunMsg.getAttribute(STUN_ATTR_XOR_MAPPED_ADDRESS).value, 'tcp'))
                )
            }),
            stunError: (cb) => stunServer.send(
                stun.createMessage(STUN_BINDING_REQUEST), 19302, 'stun.l.google.com', cb),
            peerInfo: (cb) => waterfall([
                (cb) => libp2pCrypto.keys.generateKeyPair('secp256k1', 256, cb),
                (key, cb) => {
                    let hash = crypto.createHash('sha256').update(key.public.bytes).digest()
                    let id = Multihash.encode(hash, 'sha2-256')
                    PeerInfo.create(new PeerId(id, key, key.public), cb)
                }
            ], cb)
        }, cb),
        (results, cb) => {
            results.peerInfo.multiaddrs.add('/ip4/0.0.0.0/tcp/' + port + '/ipfs/' + bs58.encode(results.peerInfo.id.pubKey.marshal()))
            // results.peerInfo.multiaddrs.add('/ip4/0.0.0.0/tcp/' + port + '/wss/p2p-webrtc-star/ipfs/' + bs58.encode(results.peerInfo.id.pubKey.marshal()))

            // results.peerInfo.multiaddrs.add('/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/ipfs/' + bs58.encode(results.peerInfo.id.pubKey.marshal()))
            this.peerInfoToString(results.peerInfo)
            console.log('Public address received from STUN server ' + results.publicAddress.toString() + '. Own port ' + port)
            cb(null, results.peerInfo, results.publicAddress)
        }
    ], cb)
}

module.exports.peerInfoToString = (peerInfo) => {
    peerInfo.multiaddrs.forEach(addr => {
        console.log(addr.toString())
    })
}

module.exports.warmUpNodes = (nodes, cb) => {
    times(nodes.length - 1, (n, cb) => {
        // console.log('trying to connect from ' + nodes[n].peerInfo.id.toB58String() + ' to ' + nodes[n + 1].peerInfo.id.toB58String())
        nodes[n].dial(nodes[n+1].peerInfo, (err, conn) => {
            if (err) { throw err }
            cb(null)
        })
    }, (err) => cb(err, nodes))
}

function generateKeyPair() {
    let privKey, publicKey

    do {
        privKey = crypto.randomBytes(Header.PRIVATE_KEY_LENGTH)
    } while (!secp256k1.privateKeyVerify(privKey))
    publicKey = secp256k1.publicKeyCreate(privKey)

    return { privKey, publicKey }
}

module.exports.generateKeyPairs = (amount) => {
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