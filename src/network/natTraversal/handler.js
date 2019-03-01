'use strict'

const SimplePeer = require('simple-peer')
const rlp = require('rlp')
const toPull = require('stream-to-pull-stream')
const { establishConnection, log } = require('../../utils')
const { PROTOCOL_WEBRTC_SIGNALING } = require('../../constants')
const PeerId = require('peer-id')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const wrtc = require('wrtc')
const Connection = require('interface-connection').Connection

module.exports = (self) => (protocol, conn) => pull(
    conn,
    lp.decode(),
    (read) => {
        let conns = new Map()
        let first
        let channel
        const messages = []
        let ended = false

        let next = () => { }
        return function foo(end, cb) {
            read(end, (end, data) => {
                if (end)
                    return cb(end)

                const decoded = rlp.decode(data)

                if (decoded.length < 2)
                    return cb()

                const recipient = new PeerId(decoded[0])
                if (!recipient.isEqual(self.sw._peerInfo.id)) {
                    let conn = conns.get(recipient.toB58String())
                    if (conn) {
                        cb(null, data)
                    } else {
                        first = true
                        establishConnection(self.sw, recipient, { protocol: PROTOCOL_WEBRTC_SIGNALING }, (err, conn) => {
                            if (err)
                                return cb(err)

                            pull(
                                lp.encode(),
                                conn,
                                lp.decode()
                            )((end, cb) => {
                                if (first) {
                                    first = false
                                    return cb(null, data)
                                }

                                foo(end, cb)
                            })(end, cb)
                        })
                    }
                } else {
                    const signalingData = JSON.parse(decoded[1])

                    if (!channel) {
                        channel = SimplePeer({
                            initiator: false,
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
                            allowHalfTrickle: true,
                            wrtc: wrtc,
                        })

                        const end = (err) => {
                            ended = true
                            if (!next.called)
                                return next(err ? err : true)
                        }

                        channel.on('connect', () => {
                            const conn = new Connection(toPull.duplex(channel))
                            conn.getObservedAddrs = (cb) => cb(null, [])
                            conn.setPeerInfo

                            setImmediate(() => {
                                self.listener.emit('connection', conn)
                            })
                            end()
                        })
                        channel.on('error', end)
                        channel.on('close', end)
                        channel.on('signal', (signalingData) => {
                            console.log('emitting', signalingData)

                            if (ended)
                                return

                            if (!next.called)
                                return cb(null, Buffer.from(JSON.stringify(signalingData)))

                            messages.push(signalingData)
                        })
                    }

                    if (ended)
                        return cb(end ? end : true)

                    next = cb

                    console.log('receiving', signalingData)

                    channel.signal(signalingData)

                    if (messages.length > 0)
                        return cb(null, Buffer.from(JSON.stringify(signalingData)))
                }
            })
        }
    },
    lp.encode(),
    conn
)
