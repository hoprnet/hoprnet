'use strict'

const UDP4 = require('./udp4')
const UDP6 = require('./udp6')

const EventEmitter = require('events').EventEmitter
const SimplePeer = require('simple-peer')
const toPull = require('stream-to-pull-stream')

const os = require('os')
const dgram = require('dgram')

const wrtc = require('wrtc')

const Connection = require('interface-connection').Connection
const Multiaddr = require('multiaddr')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')

const { answerStunRequest, getPublicIp, getSTUNServers } = require('./stun')

const mixin = Base =>
    class extends Base {
        constructor(opts) {
            super(opts)

            this.channels = new Map()
        }

        establishWebRTCConnection(msg, id, send, connHandler) {
            let channel = this.channels.get(id)

            if (!channel) {
                channel = SimplePeer({
                    initiator: false,
                    config: { iceServers: getSTUNServers(this.node.bootstrapServers) },
                    trickle: true,
                    allowHalfTrickle: true,
                    wrtc
                })

                channel.on('signal', data =>
                    send(Buffer.from(JSON.stringify(data)), (err, bytes) => {
                        if (err) console.log(err)
                        // console.log(err, bytes)
                    })
                )

                channel.on('connect', () => {
                    // console.log('[responder] connected')
                    let conn = new Connection(toPull.duplex(channel))

                    conn.getObservedAddrs = callback => callback(null, [])
                    connHandler(conn)
                })

                channel.on('close', () => {
                    // @TODO add proper handling
                    this.channels.delete(id)
                })

                channel.on('err', err => {
                    // @TODO add proper handling
                    console.log(err.message)
                    // listener.emit('err', err)
                    this.channels.delete(id)
                })

                this.channels.set(id, channel)
            }

            channel.signal(JSON.parse(msg))
        }

        dial(multiaddr, options, cb) {
            // console.log(`calling ${multiaddr.toString()}`)
            if (typeof options === 'function') {
                cb = options
                options = {}
            }

            const opts = multiaddr.toOptions()

            // TODO: use HOPR nodes instead of Google servers
            const channel = SimplePeer({
                initiator: true,
                config: { iceServers: getSTUNServers(this.node.bootstrapServers) },
                trickle: true,
                allowHalfTrickle: true,
                wrtc
            })

            const conn = new Connection(toPull.duplex(channel))

            const promise = new Promise((resolve, reject) => {
                const socket = dgram.createSocket(this.socketType)

                channel
                    .on('signal', data =>
                        socket.send(Buffer.from(JSON.stringify(data)), opts.port, opts.host, (err, bytes) => {
                            if (err) console.log(err)
                        })
                    )

                    .on('connect', async () => {
                        // console.log('[initiator] connected')

                        const peerInfo = await PeerInfo.create(await PeerId.createFromB58String(multiaddr.getPeerId()))

                        peerInfo.multiaddrs.add(multiaddr)
                        peerInfo.connect(multiaddr)

                        conn.setPeerInfo(peerInfo)
                        resolve(conn)
                    })

                    .on('error', err => {
                        console.log(err.message)
                        // reject(err)
                    })

                    .on('close', () => {
                        // console.log('closed. TODO!')
                    })

                socket.on('message', data => channel.signal(JSON.parse(data)))
            })

            if (cb) {
                promise.then(conn => cb(null, conn), cb)
                return conn
            } else {
                return promise
            }
        }

        createListener(options, connHandler) {
            if (typeof options === 'function') {
                connHandler = options
                options = {}
            }

            const listener = new EventEmitter()

            const server = dgram.createSocket(this.socketType)

            server.on('listening', () => listener.emit('listening'))
            server.on('error', err => listener.emit('error', err))
            server.on('close', () => listener.emit('close'))

            server.on('message', (msg, rinfo) => {
                if (msg[0] === '{'.charCodeAt(0)) {
                    // WebRTC requests come as JSON encoded messages
                    // thus, the msgs start with `{`
                    const id = `${rinfo.address} ${rinfo.port}`

                    this.establishWebRTCConnection(
                        msg,
                        id,
                        (msg, cb) => server.send(msg, rinfo.port, rinfo.address, cb),
                        conn => {
                            listener.emit('connection', conn)
                            connHandler(conn)
                        }
                    )
                } else if (msg[0] >> 6 == 0) {
                    // STUN requests have, as stated in RFC5389, their
                    // two most significant bits set to `0`
                    answerStunRequest(msg, rinfo, (msg, cb) => server.send(msg, rinfo.port, rinfo.address, cb))
                } else {
                    console.log(`Discarding message "${msg.toString()}" from ${rinfo.address}:${rinfo.port}`)
                }
            })

            let listeningAddr
            listener.listen = (ma, cb) =>
                new Promise((resolve, reject) => {
                    listeningAddr = ma
                    const opts = ma.toOptions()
                    server.bind(opts.port, err => {
                        if (err) return cb ? cb(err) : reject(err)

                        cb ? cb() : resolve()
                    })
                })

            listener.getAddrs = async cb => {
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

                let addrs = []

                Object.values(netInterfaces).forEach(netInterface => {
                    addrs.push(
                        ...netInterface
                            .filter(iface => !iface.internal && iface.family.toLowerCase() === this.family)
                            .map(addr =>
                                Multiaddr.fromNodeAddress({ port: serverAddr.port, ...addr }, 'udp').encapsulate(`/ipfs/${this.node.peerInfo.id.toB58String()}`)
                            )
                    )
                })

                addrs.push(this.getLocalhost(serverAddr))

                if (this.node.bootstrapServers && this.family.toLowerCase() === 'ipv4') {
                    addrs.push(await getPublicIp(this.node.bootstrapServers, this.node.peerInfo.id))
                }

                addrs = super.sortAddrs(addrs)

                return cb ? cb(null, addrs) : addrs
            }

            listener.close = () =>
                new Promise((resolve, reject) => {
                    server.close(err => {
                        if (err) return reject(err)

                        resolve()
                    })
                })

            return listener
        }

        filter(multiaddrs) {
            if (!Array.isArray(multiaddrs)) multiaddrs = [multiaddrs]

            return multiaddrs.filter(ma => ma.toOptions().family === this.family)
        }
    }

module.exports.Basev4 = mixin(UDP4)

module.exports.Basev6 = mixin(UDP6)
