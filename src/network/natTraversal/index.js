'use strict'

const EventEmitter = require('events').EventEmitter
const SimplePeer = require('simple-peer')
const toPull = require('stream-to-pull-stream')
const { establishConnection, match } = require('../../utils')
const { PROTOCOL_WEBRTC_SIGNALING } = require('../../constants')
const withIs = require('class-is')
const rlp = require('rlp')
const PeerId = require('peer-id')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const Pushable = require('pull-pushable')
const once = require('once')
const bs58 = require('bs58')
const Connection = require('interface-connection').Connection

const register = require('./register')
const handler = require('./handler')

const { waterfall, groupBy } = require('neo-async')
const wrtc = require('wrtc')

class WebRTC {
    constructor(options, sw, peerRouting) {
        this.sw = sw
        this.options = options

        if (peerRouting)
            this.peerRouting = peerRouting

        this.className = 'WebRTCStar'

        this.sw.handle(PROTOCOL_WEBRTC_SIGNALING, handler(this))
        this.sw.on('peer-mux-established', register(this))

        this.addrs = []
        this.listener = new EventEmitter()

        this.channels = []
        this.listener.listen = (multiaddrs, cb) => {
            if (Array.isArray(multiaddrs)) {
                this.addrs.push(...multiaddrs)
            } else {
                this.addrs.push(multiaddrs)
            }

            groupBy(multiaddrs, (addr, cb) => {
                const toDial = addr.decapsulate('p2p-webrtc-star').getPeerId()

                // Big TODO!!!
                establishConnection(this.sw, toDial, { peerRouting: this.peerRouting }, (err) => {
                    if (err)
                        return cb(null, 'offline')

                    cb(null, 'online')
                })
            }, (err, { online, offline }) => {
                this.sw._peerInfo.multiaddrs.replace(offline, online)

                // if (err)
                //     return setImmediate(() => {
                //         listener.emit('error')
                //         cb(err)
                //     })

                const self = this
                setImmediate(() => {
                    self.listener.emit('listening')
                    cb()
                })

            })
        }

        this.listener.getAddrs = (cb) => cb(null, addrs)
        this.listener.close = (options, cb) => {
            if (typeof options === 'function') {
                cb = options
            }

            cb = cb ? once(cb) : noop

            this.channels.forEach(channel.destroy)

            this.sw.unhandle(PROTOCOL_WEBRTC_SIGNALING)

            setImmediate(() => {
                listener.emit('close')
                cb()
            })
        }
    }

    dial(multiaddr, callback) {
        if (typeof options === 'function') {
            callback = options
            options = {}
        }

        callback = callback ? once(callback) : noop

        const channel = SimplePeer({
            initiator: true,
            //channelConfig: {},
            //channelName: '<random string>',
            //config: { iceServers: [{ urls: 'stun:stun.l.google.com:19302' }, { urls: 'stun:global.stun.twilio.com:3478?transport=udp' }] },
            //constraints: {},
            //offerConstraints: {},
            //answerConstraints: {},
            //sdpTransform: function (sdp) { return sdp },
            //stream: false,
            //streams: [],
            trickle: false,
            allowHalfTrickle: false,
            wrtc: wrtc,
        })

        const peerId = PeerId.createFromB58String(multiaddr.decapsulate('p2p-webrtc-star').getPeerId())

        const p = Pushable()

        channel.on('signal', (signalingData) => {
            p.push(
                rlp.encode([
                    Buffer.from(bs58.decode(match.WebRTC_DESTINATION(multiaddr).getPeerId())),
                    JSON.stringify(signalingData)
                ])
            )
            })

        channel.once('error', (err) => {
            p.end(err)
            channel.removeAllListeners()
            callback(err)
        })

        channel.once('timeout', () => {
            p.end()
            channel.removeAllListeners()
            callback(Error(`Timed out while trying to connect to peer ${peerId.toB58String()} through WebRTC.`))
        })

        waterfall([
            (cb) => establishConnection(this.sw, peerId, {
                protocol: PROTOCOL_WEBRTC_SIGNALING,
                // another big TODO!!!
                peerRouting: this.peerRouting
            }, cb),
            (conn, cb) => {
                let connected = false

                channel.once('connect', () => {
                    connected = true

                    return cb()
                })

                pull(
                    p,
                    lp.encode(),
                    conn,
                    lp.decode(),
                    pull.drain((data) => {
                        channel.signal(JSON.parse(data))

                        return !connected
                    })
                )
            }
        ], (err) => {
            if (err)
                return callback(err)

            p.end()

            const conn = new Connection(toPull.duplex(channel))
            conn.getObservedAddrs = () => { }

            callback(null, conn)
        })
    }

    createListener(options, handler) {
        if (typeof options === 'function') {
            handler = options
            options = {}
        }

        this.listener.on('connection', (err, conn) => {
            handler(err, conn)
        })

        return this.listener
    }

    filter(multiaddrs) {
        if (!Array.isArray(multiaddrs)) {
            multiaddrs = [multiaddrs]
        }

        return multiaddrs.filter(match.WebRTC)
    }
}

module.exports = withIs(WebRTC, {
    className: 'WebRTC',
    symbolName: '@validitylabs/hopr/WebRTC'
})