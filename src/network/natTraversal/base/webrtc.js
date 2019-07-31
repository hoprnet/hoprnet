'use strict'

const EventEmitter = require('events').EventEmitter
const SimplePeer = require('simple-peer')
const toPull = require('stream-to-pull-stream')
const os = require('os')

const wrtc = require('wrtc')

const Connection = require('interface-connection').Connection
const Multiaddr = require('multiaddr')

const dgram = require('dgram')

module.exports = class WebRTC {
    dial(multiaddr, options, cb) {
        if (typeof options === 'function') {
            cb = options
            options = {}
        }

        const opts = multiaddr.toOptions()

        const conn = new Connection()
        new Promise((resolve, reject) => {
            const socket = dgram.createSocket(this.socketType)

            // TODO: use HOPR nodes instead of Google servers
            const channel = SimplePeer({
                initiator: true,
                config: { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:global.stun.twilio.com:3478?transport=udp' }] },
                stream: false,
                trickle: true,
                wrtc: wrtc
            })

            channel
                .on('signal', data => {
                    console.log(data, opts.port, opts.host)
                    socket.send(Buffer.from(JSON.stringify(data)), opts.port, opts.host, (err, bytes) => {
                        console.log(err, bytes)
                    })
                })

                .on('connect', () => {
                    console.log('[initiator] connected')
                    conn.setInnerConn(toPull.duplex(channel))
                    resolve()
                })

                .on('error', err => {
                    console.log(err)
                    reject(err)
                })

                .on('close', () => {
                    console.log('closed. TODO!')
                })

            socket.on('message', data => channel.signal(JSON.parse(data)))
        }).then(cb, cb)

        return conn
    }

    createListener(options, connHandler) {
        if (typeof options === 'function') {
            connHandler = options
            options = {}
        }

        const channels = new Map()

        const listener = new EventEmitter()

        const server = dgram.createSocket(this.socketType)

        server.on('message', (msg, rinfo) => {
            const id = `${rinfo.address} ${rinfo.port}`

            let channel = channels.get(id)

            if (!channel) {
                // TODO: use HOPR nodes instead of Google servers
                channel = SimplePeer({
                    initiator: false,
                    config: { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:global.stun.twilio.com:3478?transport=udp' }] },
                    stream: false,
                    trickle: true,
                    wrtc: wrtc
                })

                channel.on('signal', data => {
                    server.send(Buffer.from(JSON.stringify(data)), rinfo.port, rinfo.address, (err, bytes) => {
                        console.log(err, bytes)
                    })
                })

                channel.on('connect', () => {
                    console.log('[responder] connected')
                    connHandler(new Connection(toPull.duplex(channel)))
                })

                channel.on('close', () => {
                    channels.delete(id)
                })

                channel.on('err', err => {
                    console.log(err)
                    channels.delete(id)
                })

                channels.set(id, channel)
            }

            channel.signal(JSON.parse(msg))
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

        server.on('listening', () => listener.emit('listening'))
        server.on('error', err => listener.emit('error', err))
        server.on('close', () => listener.emit('close'))

        return listener
    }

    filter(multiaddrs) {
        if (!Array.isArray(multiaddrs)) multiaddrs = [multiaddrs]

        return multiaddrs.filter(ma => ma.toOptions().family === this.family)
    }
}
