'use strict'

const EventEmitter = require('events').EventEmitter
const net = require('net')
const SimplePeer = require('simple-peer')
const toPull = require('stream-to-pull-stream')
const os = require('os')
const { PROTOCOL_WEBRTC_SIGNALING, PROTOCOL_WEBRTC_TURN } = require('../../constants')
const withIs = require('class-is')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const Heap = require('heap')
const Pushable = require('pull-pushable')
const defer = require('pull-defer')
const rlp = require('rlp')
const PeerId = require('peer-id')

const { waterfall, tryEach } = require('neo-async')
const wrtc = require('wrtc')
const chalk = require('chalk')

const Connection = require('interface-connection').Connection

const distance = require('xor-distance')
const Multiaddr = require('multiaddr')

const register = require('./register')
const handler = require('./handler')

const NUMBER_OF_SIGNALLING_SERVERS = 2
const MAX_PARALLEL_QUERIES = 4
const MAX_QUEUE_SIZE = Infinity

const MAX_HEAP_SIZE = 100

const MAX_CONNECTIONS = 3 // TODO

class WebRTC {
    constructor(opts) {
        this.heap = []

        this.tag = 'WebRTC'

        this.node = opts.libp2p
        this.node.on('peer:discovered', this.discoveryListener)
        this.node.handle(PROTOCOL_WEBRTC_TURN, this.handler)
    }

    /**
     * Checks whether the distance of the given node to this node is smaller than all
     * previously seen nodes and if that is the case, it puts that node in the heap.
     * 
     * @param {PeerInfo} peerInfo the newly discovered node
     */
    discoveryListener(peerInfo) {
        if (heap.length <= MAX_CONNECTIONS) {
            Heap.push(this.heap, peerInfo, cmp)
            return
        }

        const distanceFarthest = distance(this.heap[this.heap.length - 1].id.toBytes(), this.node.peerInfo.id.toBytes())
        const distanceCurrent = distance(peerInfo.id.toBytes(), this.node.peerInfo.id.toBytes())

        if (distance.gt(distanceCurrent, distanceFarthest))
            return

        this.heap = Heap.pushpop(this.heap, peerInfo, (a, b) => distance.compare(
            distance(a.id.toBytes(), this.node.peerInfo.id.toBytes()),
            distance(b.id.toBytes(), this.node.peerInfo.id.toBytes())
        ))
    }

    /**
     * Establishes a relayed connection to the given over node through the closest
     * known node.
     * 
     * @async
     * @param {PeerId} destination PeerId of the node with whom the connection is
     * going to be established.
     * @return {Promise<Connection>} a connection through the relay to the destination
     */
    establishCircuit(destination) {
        return new Promise((resolve, reject) => {
            const deferred = defer.duplex()

            const relayedConn = new Connection({
                sink: deferred.sink,
                source: pull(
                    deferred.source,
                    pull.map((data) => rlp.encode([destination.toBytes(), data]))
                )
            })

            const queryNode = (peerInfo) => new Promise((resolve, reject) =>
                tryEach([
                    (cb) => this.node.dial(current[0], PROTOCOL_WEBRTC_SIGNALING, cb),
                    (cb) => waterfall([
                        (cb) => this.node.peerRouting.findPeer(current[0].id, cb),
                        (peerInfo, cb) => this.node.dial(peerInfo, PROTOCOL_WEBRTC_SIGNALING, cb)
                    ], cb)
                ], (err, conn) => {
                    if (err || !conn)
                        return reject(err ? err : Error(`Unable to connect to ${chalk.blue(peerInfo.id.toB58String())}.`))

                    try {
                        pull(
                            pull.once(destination.toBytes()),
                            lp.encode(),
                            conn,
                            lp.decode(),
                            pull.take(1),
                            pull.collect((err, answer) => {
                                if (err)
                                    return reject(err)

                                try {
                                    answer = rlp.decode(answer[0])
                                } catch (err) {
                                    reject(err)
                                }

                                const result = {
                                    connected: answer[0],
                                    conn: null,
                                    closerPeer: null
                                }

                                if (answer[0])
                                    result.conn = conn

                                if (answer.length > 1)
                                    result.closerPeer = answer[1]

                                resolve(result)
                            })
                        )
                    } catch (err) {
                        reject(err)
                    }
                })
            )

            const heap = []

            const comparator = (a, b) => distance.compare(
                distance(a.id.toBytes(), destination.toBytes()),
                distance(b.id.toBytes(), destination.toBytes())
            )

            let finished = false

            const processResults = (closestPeers) => {
                if (finished)
                    return

                closestPeers.forEach((peerInfo) => Heap.push(heap, peerInfo, comparator))

                while (heap.length > 0 && queue.getPendingLength() <= MAX_PARALLEL_QUERIES) {
                    this.queue.add(() => queryNode(Heap.pop(heap, comparator)))
                        .then(({ connected, closerPeer, conn }) => {
                            if (connected) {
                                finished = true
                                relayedConn.resolve(conn)
                                resolve(relayedConn)
                            } else {
                                processResults([closerPeer])
                            }
                        })
                        .catch((err) => processResults())
                }

                if (heap.length == 0 && queue.getPendingLength() == 0 && queue.getQueueLength() == 0)
                    return reject(Error(`Unable to create circuit to peer ${chalk.blue(destination.toB58String())}.`))
            }
        })
    }

    handler(protocol, conn) {
        const reply = Pushable()
        const relayedMessages = Pushable()

        const relayConn = new Connection()

        pull(
            relayedMessages,
            lp.encode(),
            relayConn,
            lp.decode(),
            pull.drain(
                (data) => reply.push(data),
                () => {
                    relayedMessages.end()
                    reply.end()
                })
        )

        pull(
            conn,
            lp.decode(),
            pull.drain((data) => {
                const [destination, payload] = rlp.decode(data)

                this.node.dial(PeerId.createFromBytes(destination), { noRelay: true }, (err, conn) => {
                    if (err || !conn) {
                        reply.push(rlp.encode([false, this.getClosestPeers(1, PeerId.createFromBytes(destination))]))
                        // reply.end()
                        return
                    }

                    reply.push(rlp.encode([true]))
                    relayConn.setInnerConn(conn)
                    relayedMessages.push(payload)
                })
            })
        )

        pull(
            reply,
            lp.encode(),
            conn
        )
    }

    getClosestPeers(desiredAmountOfPeers, destination) {
        const cmp = () => distance.compare(
            distance(a.id.toBytes(), destination.toBytes()),
            distance(b.id.toBytes(), destination.toBytes())
        )

        return Heap.nsmallest(this.node.peerBook.getAllArray(), desiredAmountOfPeers, cmp)
    }

    dial(multiaddr, options, cb) {
        if (typeof options === 'function') {
            cb = options
            options = {}
        }

        const conn = new Connection()
        new Promise((resolve, reject) => {
            let circuitConn = defer.duplex()
            let circuitConnResolved = false
            const p = Pushable()

            let channel
            pull(
                p,
                lp.encode(),
                circuitConn,
                lp.decode(),
                pull.drain((data) => channel.signal(JSON.parse(data)))
            )

            channel = SimplePeer({
                initiator: true,
                config: { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:global.stun.twilio.com:3478?transport=udp' }] },
                stream: false,
                trickle: true,
                wrtc: wrtc,
            })
                .on('error', (err) => {
                    p.end()
                    console.log(err)
                    conn.setInnerConn(circuitConn)
                    resolve()
                })
                .on('close', () => {
                    p.end()
                })
                .on('connect', () => {
                    p.end()
                    conn.setInnerConn(toPull.duplex(channel))
                    resolve()
                })
                .on('signal', (data) => {
                    if (!circuitConnResolved) {
                        circuitConnResolved = true
                        try {
                            const opts = multiaddr.toOptions()
                            const socket = net.connect(opts.port, opts.host)
                                .on('error', reject)
                                .on('connect', () => circuitConn.resolve(new Connection(toPull.duplex(socket))))
                        } catch (err) {
                            console.log(err)
                        }
                        if (!options.noRelay) {
                            // try {
                            //     circuitConn.resolve(await establishCircuit(multiaddr.getPeerId()))
                            // } catch (err) {
                            //     channel.destroy()
                            //     return reject(err)
                            // }
                        }
                    }

                    p.push(Buffer.from(JSON.stringify(data)))
                })
        })
            .then(cb, cb)
        return conn
    }

    createListener(options, connHandler) {
        if (typeof options === 'function') {
            connHandler = options
            options = {}
        }
        const listener = new EventEmitter()

        const server = net.createServer((socket) => {
            socket.on('error', () => { })

            const conn = new Connection(toPull.duplex(socket))
            const p = Pushable()

            let channel
            pull(
                conn,
                lp.decode(),
                pull.drain((data) => channel.signal(JSON.parse(data)))
            )

            pull(
                p,
                lp.encode(),
                conn
            )

            channel = SimplePeer({
                initiator: false,
                config: { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:global.stun.twilio.com:3478?transport=udp' }] },
                trickle: true,
                wrtc: wrtc,
            })
                .on('signal', (data) => p.push(Buffer.from(JSON.stringify(data))))
                .on('connect', () => connHandler(new Connection(toPull.duplex(channel))))
        })

        server.on('listening', () => listener.emit('listening'))
        server.on('error', (err) => listener.emit('error', err))
        server.on('close', () => listener.emit('close'))

        let listeningAddr
        listener.listen = (ma, cb) => new Promise((resolve, reject) => {
            listeningAddr = ma
            const opts = ma.toOptions()
            server.listen(opts.port, (err) => {
                if (err)
                    return cb ? cb(err) : reject(err)

                cb ? cb() : resolve()
            })
        })

        listener.getAddrs = (cb) => {
            const serverAddr = server.address()

            if (!serverAddr) {
                const err = Error('Listener is not ready yet')
                if (cb) {
                    return cb(err)
                } else {
                    throw err
                }
            }

            const netInterfaces = os.networkInterfaces()

            const addrs = Object.values(netInterfaces)
                .reduce((acc, netInterface) => {
                    const external = netInterface
                        .filter((iface) => !iface.internal)
                        .map((addr) => Multiaddr
                            .fromNodeAddress({ port: serverAddr.port, ...addr }, 'tcp')
                            .encapsulate(`/ipfs/${listeningAddr.getPeerId()}`))

                    acc.push(...external)

                    return acc
                }, [])

            return cb ? cb(null, addrs) : addrs
        }

        listener.close = () => new Promise((resolve, reject) => {
            server.close((err) => {
                if (err)
                    return reject(err)

                resolve()
            })
        })

        return listener
    }

    filter(multiaddrs) {
        if (!Array.isArray(multiaddrs))
            multiaddrs = [multiaddrs]

        return multiaddrs
        // return multiaddrs.filter(match.WebRTC)
    }
}

module.exports = withIs(WebRTC, {
    className: 'WebRTC',
    symbolName: '@validitylabs/hopr/WebRTC'
})