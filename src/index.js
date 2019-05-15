'use strict'

const libp2p = require('libp2p')
const MPLEX = require('libp2p-mplex')
const KadDHT = require('libp2p-kad-dht')
const SECIO = require('libp2p-secio')
const TCP = require('libp2p-tcp')
const WebRTC = require('./network/natTraversal')

const defaultsDeep = require('@nodeutils/defaults-deep')
const once = require('once')

const { createPacket } = require('./packet')
const registerHandlers = require('./handlers')
const { NAME, PACKET_SIZE, PROTOCOL_STRING, MAX_HOPS } = require('./constants')
const crawlNetwork = require('./network/crawl')
const heartbeat = require('./network/heartbeat')
const getPubKey = require('./getPubKey')
const getPeerInfo = require('./getPeerInfo')
const { randomSubset, serializePeerBook, deserializePeerBook, log, match } = require('./utils')

const fs = require('fs')
const levelup = require('levelup')
const leveldown = require('leveldown')

const PeerId = require('peer-id')
const PeerInfo = require('peer-info')
const PeerBook = require('peer-book')
const Multiaddr = require('multiaddr')

const PaymentChannels = require('./paymentChannels')
const PublicIp = require('./network/natTraversal/stun')

const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const { waterfall, times, parallel, map } = require('neo-async')
const Acknowledgement = require('./acknowledgement')

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
                    TCP,
                    WebRTC
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
                dht: KadDHT,
                /**
                 * Necessary to use WebRTC (and to support proper NAT traversal)
                 */
                // peerDiscovery: [
                //     WebRTC.discovery
                // ]
            },
            config: {
                // peerDiscovery: {
                //     webRTCStar: {
                //         enabled: true
                //     }
                // },
                dht: {
                    enabled: true
                },
                relay: {
                    enabled: false
                }
            }
        }
        super(defaultsDeep(_options, defaults))

        this.db = db

        // Functionality to ask another node for its public key in case that the
        // public key is not available which is not necessary anymore.
        //
        // Notice: don't forget to activate the corresponding handler in `handlers/index.js`
        //
        this.getPubKey = getPubKey(this)
        // this.getPubKey = this._dht.getPublicKey
    }

    /**
     * Creates a new node and invokes @param cb with `(err, node)` when finished.
     * 
     * @param {Object} options the parameters
     * @param {Object} options.web3provider a web3 provider, default `http://localhost:8545`
     * @param {String} options.contractAddress the Ethereum address of the contract
     * @param {Object} options.peerInfo 
     * @param {Function} cb callback when node is ready
     */
    static async createNode(options, cb) {
        return new Promise(async (resolve, reject) => {
            if (typeof options === 'function') {
                cb = options
                options = {}
            }

            options.output = options.output || console.log

            const db = Hopr.openDatabase(`${process.cwd()}/db`, options)

            // peerBook: (cb) => {
            //     if (options.peerBook) {
            //         cb(null, options.peerBook)
            //     } else {
            //         Hopr.importPeerBook(db, cb)
            //     }
            // }

            const hopr = new Hopr({
                config: {
                    // WebRTC: options.WebRTC
                },
                // peerBook: peerBook,
                peerInfo: await getPeerInfo(options, db)
            }, db)

            hopr.start(options, (err) => {
                if (err)
                    return reject(err)

                resolve(hopr)
            })
        })



    }

    /**
     * This method starts the node and registers all necessary handlers. It will
     * also open the database and creates one if it doesn't exists.
     * 
     * @param {Function} output function to which the plaintext of the received message is passed
     * @param {Function} cb callback when node is ready
     */
    start(options, cb) {
        waterfall([
            (cb) => super.start(cb),
            (cb) => {
                registerHandlers(this, options)

                this.bootstrapServers = process.env.BOOTSTRAP_SERVERS.split(',').map(addr => Multiaddr(addr))
                this.heartbeat = heartbeat(this)
                this.getPublicIp = PublicIp(this, options)
                this.crawlNetwork = crawlNetwork(this, options.Crawler || {})

                this.peerInfo.multiaddrs.forEach((addr) => {
                    if (match.LOCALHOST(addr)) {
                        this.peerInfo.multiaddrs.delete(addr)
                    }
                })

                return cb()
            },
            (cb) => parallel({
                publicAddrs: (cb) => {
                    if (options['bootstrap-node'] || process.env.DEMO || !this.bootstrapServers || this.bootstrapServers.length == 0)
                        return cb()

                    this.getPublicIp(cb)
                },
                paymentChannels: (cb) => {
                    if (options['bootstrap-node'])
                        return cb()

                    PaymentChannels.create(this, cb)
                }
            }, cb),
            ({ paymentChannels, publicAddrs }, cb) => {
                this.paymentChannels = paymentChannels

                if (publicAddrs)
                    publicAddrs.forEach((addr) => this.peerInfo.multiaddrs.add(addr.encapsulate(`/${NAME}/${this.peerInfo.id.toB58String()}`)))

                if (!options['bootstrap-node'])
                    this.paymentChannels = paymentChannels

                return cb(null, this)
            }
        ], cb)
    }

    /**
     * Shutdown the node and saves keys and peerBook in the database
     * @param {Function} cb 
     */
    stop(cb = () => { }) {
        log(this.peerInfo.id, `Shutting down...`)

        clearInterval(this.heartbeat)

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
     * @param {PeerId | PeerInfo | String} destination PeerId of the destination
     * @param {Function} cb called with `(err)` in case of an error, or when receiving 
     * the acknowledgement of the first hop
     */
    sendMessage(msg, destination, cb) {
        if (!msg)
            return cb(Error(`Expecting a non-empty message.`))

        if (!destination)
            return cb(Error(`Expecting a non-empty destination.`))

        if (PeerInfo.isPeerInfo(destination))
            destination = destination.id

        if (typeof destination === 'string')
            destination = PeerId.createFromB58String(destination)

        if (!PeerId.isPeerId(destination))
            return cb(Error(`Unable to parse given destination to a PeerId instance. Got type ${typeof destination} with value ${destination}.`))

        // Let's try to convert input msg to a Buffer in case it isn't already a Buffer
        if (!Buffer.isBuffer(msg)) {
            switch (typeof msg) {
                default:
                    return cb(Error(`Invalid input value. Got '${typeof msg}'.`))
                case 'number': msg = msg.toString()
                case 'string': msg = Buffer.from(msg)
            }
        }

        cb = cb ? once(cb) : () => { }

        times(Math.ceil(msg.length / PACKET_SIZE), (n, cb) =>
            waterfall([
                (cb) => this.getIntermediateNodes(destination, cb),
                (intermediateNodes, cb) => map(intermediateNodes.concat(destination), this.getPubKey, cb),
                (intermediateNodes, cb) => parallel({
                    conn: (cb) => waterfall([
                        (cb) => this.peerRouting.findPeer(intermediateNodes[0].id, cb),
                        (peerInfo, cb) => this.dialProtocol(peerInfo, PROTOCOL_STRING, cb),
                    ], cb),
                    packet: (cb) => createPacket(
                        this,
                        msg.slice(n * PACKET_SIZE, Math.min(msg.length, (n + 1) * PACKET_SIZE)),
                        intermediateNodes.map(peerInfo => peerInfo.id),
                        cb
                    )
                }, cb),
                (results, cb) => pull(
                    pull.once(results.packet.toBuffer()),
                    lp.encode(),
                    results.conn,
                    lp.decode({
                        maxLength: Acknowledgement.SIZE
                    }),
                    pull.take(1),
                    pull.drain((data) => {
                        log(this.peerInfo.id, `Received acknowledgement.`)
                        // return cb()
                        // if (!cb.called) {
                        //     return cb()
                        // }
                    })
                )
            ], cb),
            (err) => {
                if (err)
                    console.log(err)

                return cb(err)
            })
    }

    /**
     * Takes a destination and samples randomly intermediate nodes
     * that will relay that message before it reaches its destination.
     * 
     * @param {Object} destination instance of peerInfo that contains the peerId of the destination 
     * @param {Function} cb the function that called afterwards
     */
    getIntermediateNodes(destination, cb) {
        const filter = (peerInfo) =>
            !peerInfo.id.isEqual(this.peerInfo.id) &&
            !peerInfo.id.isEqual(destination) &&
            !this.bootstrapServers.some((multiaddr) => PeerId.createFromB58String(multiaddr.getPeerId()).isEqual(peerInfo.id))

        this.crawlNetwork((err) => {
            if (err)
                return cb(err)

            cb(null, randomSubset(
                this.peerBook.getAllArray(), MAX_HOPS - 1, filter).map((peerInfo) => peerInfo.id)
            )
        })
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
                deserializePeerBook(value, peerBook, (err) => {
                    if (err) {
                        cb(err)
                    } else {
                        cb(null, peerBook)
                    }
                })
            }
        })
    }

    exportPeerBook(cb) {
        const key = 'peer-book'

        this.db.put(key, serializePeerBook(this.peerBook), cb)
    }

    static openDatabase(db_dir, options) {
        try {
            fs.accessSync(db_dir)
        } catch (err) {
            if (!err.code === 'ENOENT' && !err.code === 'EEXIST')
                throw err

            fs.mkdirSync(db_dir, {
                // TODO: Change to something better
                mode: 0o777
            })
        }

        if (options && Number.isInteger(options.id)) {
            // Only for unit testing !!!
            db_dir = `${db_dir}/node ${options.id}`

            try {
                fs.accessSync(db_dir)
            } catch (err) {
                if (!err.code === 'ENOENT' && !err.code === 'EEXIST')
                    throw err

                fs.mkdirSync(db_dir, {
                    // TODO: Change to something better
                    mode: 0o777
                })
            }
            //     clearDirectory(db_dir)
            //     fs.mkdirSync(db_dir, {
            //         mode: 0o777
            //     })
            // --------------------------
        }

        return levelup(leveldown(db_dir))
    }
}

module.exports = Hopr