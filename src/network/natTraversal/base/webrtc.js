'use strict'

const EventEmitter = require('events').EventEmitter
const SimplePeer = require('simple-peer')
const toPull = require('stream-to-pull-stream')
const os = require('os')

const wrtc = require('wrtc')

const Connection = require('interface-connection').Connection
const Multiaddr = require('multiaddr')

const PeerInfo = require('peer-info')
const PeerId = require('peer-id')

const dgram = require('dgram')

const stun = require('webrtc-stun')

module.exports = class WebRTC {
    constructor() {
        this.channels = new Map()
    }

    dial(multiaddr, options, cb) {
        console.log(`calling ${multiaddr.toString()}`)
        if (typeof options === 'function') {
            cb = options
            options = {}
        }

        const opts = multiaddr.toOptions()

        // TODO: use HOPR nodes instead of Google servers
        const channel = SimplePeer({
            initiator: true,
            config: { iceServers: prepareBootstrapServers(this.node.bootstrapServers) },
            trickle: true,
            allowHalfTrickle: true,
            wrtc
        })

        const conn = new Connection(toPull.duplex(channel))

        new Promise((resolve, reject) => {
            const socket = dgram.createSocket(this.socketType)

            channel
                .on('signal', data => {
                    socket.send(Buffer.from(JSON.stringify(data)), opts.port, opts.host, (err, bytes) => {
                        if (err) console.log(err)
                    })
                })

                .on('connect', async () => {
                    console.log('[initiator] connected')

                    const peerInfo = await PeerInfo.create(await PeerId.createFromB58String(multiaddr.getPeerId()))

                    peerInfo.multiaddrs.add(multiaddr)
                    peerInfo.connect(multiaddr)

                    conn.setPeerInfo(peerInfo)
                    resolve(conn)
                })

                .on('error', err => {
                    console.log(err)
                    reject(err)
                })

                .on('close', () => {
                    // console.log('closed. TODO!')
                })

            socket.on('message', data => channel.signal(JSON.parse(data)))
        }).then(conn => cb(null, conn), cb)

        return conn
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

        const establishWebRTCConnection = (msg, rinfo) => {
            const id = `${rinfo.address} ${rinfo.port}`

            let channel = this.channels.get(id)

            if (!channel) {
                // TODO: use HOPR nodes instead of Google servers
                channel = SimplePeer({
                    initiator: false,
                    config: { iceServers: prepareBootstrapServers(this.node.bootstrapServers) },
                    trickle: true,
                    allowHalfTrickle: true,
                    wrtc
                })

                let conn = new Connection(toPull.duplex(channel))

                channel.on('signal', data => {
                    server.send(Buffer.from(JSON.stringify(data)), rinfo.port, rinfo.address, (err, bytes) => {
                        if (err) console.log(err)
                        // console.log(err, bytes)
                    })
                })

                channel.on('connect', () => {
                    console.log('[responder] connected')
                    conn.getObservedAddrs = callback => callback(null, [])
                    listener.emit('connection', conn)
                    connHandler(conn)
                })

                channel.on('close', () => {
                    this.channels.delete(id)
                })

                channel.on('err', err => {
                    console.log(err)
                    this.channels.delete(id)
                })

                this.channels.set(id, channel)
            }

            channel.signal(JSON.parse(msg))
        }

        const answerStunRequest = (msg, rinfo) => {
            const req = stun.createBlank()

            // if msg is valid STUN message
            if (req.loadBuffer(msg)) {
                // if STUN message is BINDING_REQUEST and valid content
                if (req.isBindingRequest({ fingerprint: true })) {
                    // console.log('REQUEST', req)

                    const res = req
                        .createBindingResponse(true)
                        .setXorMappedAddressAttribute(rinfo)
                        .setFingerprintAttribute()

                    // console.log('RESPONSE', res)
                    server.send(res.toBuffer(), rinfo.port, rinfo.address)
                }
            }
        }

        server.on('message', (msg, rinfo) => {
            console.lot(msg + " ---- " + msg.toString())
            if (msg[0] === '{'.charCodeAt(0)) {
                // WebRTC requests come as JSON encoded messages
                // thus, the msg starts with `{`
                establishWebRTCConnection(msg, rinfo)
            } else if (msg[0] >> 6 == 0) {
                // STUN requests have, as stated in RFC5389, their
                // two most significant bits set to `0`
                answerStunRequest(msg, rinfo)
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

        listener.getAddrs = cb => {
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

            const addrs = Object.values(netInterfaces).reduce((acc, netInterface) => {
                const externalAddrs = netInterface
                    .filter(iface => !iface.internal && iface.family.toLowerCase() === this.family)
                    .map(addr => Multiaddr.fromNodeAddress({ port: serverAddr.port, ...addr }, 'udp').encapsulate(`/ipfs/${listeningAddr.getPeerId()}`))

                acc.push(...externalAddrs)

                return acc
            }, [])

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

function prepareBootstrapServers(peerInfos) {
    const result = []
    peerInfos.forEach(peerInfo => {
        peerInfo.multiaddrs.forEach(ma => {
            const opts = ma.toOptions()
            result.push({
                urls: `stun:${opts.host}:${opts.port}`
            })
        })
    })

    return result
}
