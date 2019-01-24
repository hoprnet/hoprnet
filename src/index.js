'use strict'

const libp2p = require('libp2p')
const TCP = require('libp2p-tcp')
const MPLEX = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
const SECIO = require('libp2p-secio')
const defaultsDeep = require('@nodeutils/defaults-deep')

const Keychain = require('libp2p-keychain')
const FsStore = require('datastore-fs')

const { createPacket } = require('./packet')
const registerHandlers = require('./handlers')
const c = require('./constants')
const crawlNetwork = require('./crawlNetwork')
const getPubKey = require('./getPubKey')
const getPeerInfo = require('./getPeerInfo')
const { randomSubset, serializePeerBook, deserializePeerBook, log, clearDirectory } = require('./utils')
const PendingTransactions = require('./pendingTransactions')

// const wrtc = require('wrtc')
const WStar = require('libp2p-webrtc-star')
// const WebRTC = new WStar({
//     wrtc: wrtc
// })

const fs = require('fs')
const levelup = require('levelup')
const leveldown = require('leveldown')

const PeerId = require('peer-id')
const PeerInfo = require('peer-info')
const PeerBook = require('peer-book')
const Multiaddr = require('multiaddr')
const { resolve } = require('path')


const PaymentChannels = require('./paymentChannels')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { waterfall, times, parallel, map } = require('neo-async')

// const BOOTSTRAP_NODE = Multiaddr('/ip4/127.0.0.1/tcp/9090/')

class Hopr extends libp2p {
    /**
     * @constructor
     * 
     * @param {Object} _options 
     * @param {Object} provider 
     */
    constructor(_options, db) {
        if (!_options || !_options.peerInfo || !PeerInfo.isPeerInfo(_options.peerInfo))
            throw Error('Invalid input parameters. Expected a valid PeerInfo, but got \'' + typeof _options.peerInfo + '\' instead.')

        const defaults = {
            // The libp2p modules for this libp2p bundle
            modules: {
                /**
                 * The transport modules to use.
                 */
                transport: [
                    TCP // WebRTC
                ],
                /**
                 * To support bidirectional connection, we need a stream muxer.
                 */
                streamMuxer: [
                    MPLEX
                ],
                /**
                 * Let's have TLS-alike encrypted connections between the nodes.
                 */
                connEncryption: [
                    SECIO
                ],
                /**
                 * Necessary to have DHT lookups
                 */
                dht: KadDHT
                /**
                 * Necessary to use WebRTC (and to support proper NAT traversal)
                 */
                // peerDiscovery: [WebRTC.discovery]
            },
            config: {
                EXPERIMENTAL: {
                    // libp2p DHT implementation is still hidden behind a flag
                    dht: true
                }
            }

        }
        super(defaultsDeep(_options, defaults))

        this.db = db
        this.seenTags = new Set()
        this.crawlNetwork = crawlNetwork(this)
        this.getPubKey = getPubKey(this)
        this.pendingTransactions = new PendingTransactions(this.db)
    }

    /**
     * Creates a new node and invokes @param cb with `(err, node)` when finished.
     * 
     * @param {Object} options the parameters
     * @param {Object} options.web3provider a web3 provider, default `http://localhost:8545`
     * @param {String} options.contractAddress the Ethereum address of the contract
     * @param {Object} options.peerInfo 
     * @param {Object} options.peerBook a PeerBook instance, otherwise try to retrieve peerBook from database, otherwise create a new one
     * @param {Function} cb callback when node is ready
     */
    static createNode(options, cb) {
        if (typeof options === 'function') {
            cb = options
            options = {}
        }

        options.output = options.output || console.log

        let db_dir = resolve(__dirname, '../db')
        waterfall([
            (cb) => fs.access(db_dir, (err) => {
                if (err && !err.code === 'ENOENT' && !err.code === 'EEXIST') {
                    throw er
                } else if (err && err.code === 'ENOENT') {
                    try {
                        fs.mkdirSync(db_dir, {
                            mode: 0o777
                        })
                    } catch (err) {
                        
                    }
                    
                }

                if (options.id) {
                    // Only for unit testing !!!
                    db_dir = resolve(db_dir, `./${options.id}`)
                    delete options.id
                    fs.access(db_dir, (err) => {
                        if (err && !err.code === 'ENOENT') {
                            throw err
                        } else if (err && err.code === 'ENOENT') {
                            fs.mkdirSync(db_dir, {
                                mode: 0o777
                            })
                        }
                        // else {
                        //     clearDirectory(db_dir)
                        //     fs.mkdirSync(db_dir, {
                        //         mode: 0o777
                        //     })
                        // }
                        levelup(leveldown(db_dir), cb)
                    })
                    // --------------------------
                } else {
                    levelup(leveldown(db_dir), cb)
                }
            }),
            (db, cb) => parallel({
                peerBook: (cb) => Hopr.importPeerBook(db, cb),
                peerInfo: (cb) => PeerInfo.isPeerInfo(options.peerInfo) ? cb(null, options.peerInfo) : getPeerInfo(db, cb)
            }, (err, { peerBook, peerInfo }) => {
                if (err)
                    cb(err)

                const hopr = new Hopr({
                    peerBook: peerBook,
                    peerInfo: peerInfo
                }, db)

                hopr.start(options, cb)
            })
        ], cb)
    }

    /**
     * This method starts the node and registers all necessary handlers. It will
     * also open the database and creates one if it doesn't exists.
     * 
     * @param {Function} output function to which the plaintext of the received message is passed
     * @param {Function} cb callback when node is ready
     */
    start(options, cb) {
        parallel({
            node: (cb) => super.start(cb),
            paymentChannels: (cb) => PaymentChannels.create({
                node: this,
                provider: options.provider,
                contractAddress: options.contractAddress
            }, cb)
        }, (err, results) => {
            if (err)
                cb(err)

            registerHandlers(this, options.output)

            this.paymentChannels = results.paymentChannels

            cb(null, this)
        })
    }

    /**
     * Shutdown the node and saves keys and peerBook in the database
     * @param {Function} cb 
     */
    stop(cb = () => { }) {
        log(this.peerInfo.id, `Shutting down...`)
        waterfall([
            (cb) => this.exportPeerBook(cb),
            (cb) => super.stop(cb),
            (cb) => this.db.close(cb)
        ], cb)
    }

    /**
     * Sends a message.
     * 
     * @notice THIS METHOD WILL SPEND YOUR ETHER.
     * @notice This method will fail if there are not enough funds to open
     * the required payment channels. Please make sure that there are enough
     * funds controlled by the given key pair.
     * 
     * @param {Number | String | Buffer} msg message to send
     * @param {Object} destination PeerId of the destination
     * @param {Function} cb function to call when finished
     */
    sendMessage(msg, destination, cb) {
        if (!msg)
            throw Error('Expecting non-empty message.')

        // Let's try to convert input msg to a Buffer in case it isn't already a Buffer
        if (!Buffer.isBuffer(msg)) {
            switch (typeof msg) {
                default:
                    throw Error('Invalid input value. Got \"' + typeof msg + '\".')
                case 'number': msg = msg.toString()
                case 'string': msg = Buffer.from(msg)
            }
        }

        times(Math.ceil(msg.length / c.PACKET_SIZE), (n, cb) => {
            let path

            waterfall([
                (cb) => this.getIntermediateNodes(destination, cb),
                (intermediateNodes, cb) => {
                    path = intermediateNodes.map(peerInfo => peerInfo.id).concat(destination)

                    this.peerRouting.findPeer(path[0], cb)
                },
                (peerInfo, cb) => parallel({
                    conn: (cb) => this.dialProtocol(peerInfo, c.PROTOCOL_STRING, cb),
                    packet: (cb) => createPacket(
                        this,
                        msg.slice(n * c.PACKET_SIZE, Math.min(msg.length, (n + 1) * c.PACKET_SIZE)),
                        path,
                        cb
                    )
                }, cb),
                ({ conn, packet }, cb) => {
                    pull(
                        pull.once(packet.toBuffer()),
                        lp.encode(),
                        conn
                    )
                    cb()
                }
            ], cb)
        }, cb)
    }

    /**
     * Takes a destination and samples randomly intermediate nodes
     * that will relay that message before it reaches its destination.
     * 
     * @param {Object} destination instance of peerInfo that contains the peerId of the destination 
     * @param {Function} cb the function that called afterwards
     */
    getIntermediateNodes(destination, cb) {
        const comparator = (peerInfo) =>
            this.peerInfo.id.id.compare(peerInfo.id.id) !== 0 &&
            destination.id.compare(peerInfo.id.id) !== 0

        this.crawlNetwork(() => {
            cb(null, randomSubset(
                this.peerBook.getAllArray(), c.MAX_HOPS - 1, comparator))
        }, comparator)
    }

    static importPeerBook(db, cb) {
        const key = 'peer-book'

        const peerBook = new PeerBook()

        db.get(key, (err, value) => {
            if (err && !err.notFound) {
                cb(err)
            } else if (err && err.notFound) {
                cb(null, peerBook)
            } else {
                deserializePeerBook(value, peerBook, cb)
            }
        })
    }

    exportPeerBook(cb) {
        const key = 'peer-book'

        this.db.put(key, serializePeerBook(this.peerBook), cb)
    }
}

module.exports = Hopr