'use strict'

const withIs = require('class-is')

const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const MPLEX = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
const SECIO = require('libp2p-secio')
const defaultsDeep = require('@nodeutils/defaults-deep')


const Eth = require('web3-eth')


const Packet = require('./packet')
const registerHandlers = require('./handlers')
const c = require('./constants')
const crawlNetwork = require('./crawlNetwork')
const getPubKey = require('./getPubKey')
const getPeerInfo = require('./getPeerInfo')

// DEMO
const { randomBytes } = require('crypto')
const { bufferToNumber, randomSubset } = require('./utils')
// END DEMO



// const wrtc = require('wrtc')
const WStar = require('libp2p-webrtc-star')
// const WebRTC = new WStar({
//     wrtc: wrtc
// })

const PaymentChannels = require('./paymentChannels')


const pull = require('pull-stream')
const { waterfall, times } = require('async')

// const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')

class Hopper extends libp2p {
    constructor(_options, provider, contract) {
        const defaults = {
            // The libp2p modules for this libp2p bundle
            modules: {
                transport: [
                    TCP
                ],
                streamMuxer: [
                    MPLEX
                ],
                connEncryption: [
                    SECIO
                ],
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

        // Maybe this is not necessary
        this.eth = new Eth(provider)

        this.seenTags = new Set()
        this.pendingTransactions = new Map()
        this.paymentChannels = new PaymentChannels(this, contract)
        this.crawlNetwork = crawlNetwork(this)
        this.getPubKey = getPubKey(this)

        // const findPeer = this.peerRouting.findPeer
        // const self = this
        // this.peerRouting.findPeer = function (peerId, cb) {
        //     console.log('[\'' + self.peerInfo.id.toB58String() + '\']: Searching for \'' + peerId.toB58String() + '\'.')
        //     findPeer(peerId, (err, peerInfo) => {
        //         if (err)
        //             console.log(err)

        //         console.log('[\'' + self.peerInfo.id.toB58String() + '\']: Found peer')
        //         cb(err, peerInfo)
        //     })
        // }
    }

    static startNode(provider, output, contract, cb, peerInfo) {
        let node

        waterfall([
            (cb) => {
                if (peerInfo) {
                    cb(null, peerInfo)
                } else {
                    getPeerInfo(null, cb)
                }
            },
            (peerInfo, cb) => {
                node = new Hopper({
                    peerInfo: peerInfo
                }, provider, contract)
                cb(null)
            },
            (cb) => node.start(cb),
            (cb) => registerHandlers(node, output, cb),
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
    this.sendMessage('HelloWorld! ' + Date.now().toString(), this.peerBook.getAllArray()[bufferToNumber(randomBytes(4)) % (this.peerBook.getAllArray().length)])
}

module.exports = withIs(Hopper, { className: 'hopper', symbolName: '@validitylabs/hopper/hopper' })