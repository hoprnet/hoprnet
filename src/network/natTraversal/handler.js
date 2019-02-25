'use strict'

const SimplePeer = require('simple-peer')
const rlp = require('rlp')
const toPull = require('stream-to-pull-stream')
const { establishConnection } = require('../../utils')
const { PROTOCOL_WEBRTC_SIGNALING } = require('../../constants')
const PeerId = require('peer-id')
const paramap = require('pull-paramap')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')

module.exports = (self) => (protocol, conn) => pull(
    conn,
    lp.decode(),
    paramap((data, cb) => {
        const decoded = rlp.decode(data)
        if (decoded.length < 2)
            return cb()

        if (decoded[0].toString() == this.sw.peerInfo.id.toB58String()) {
            const channel = SimplePeer({
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
                allowHalfTrickle: false,
                wrtc: wrtc,
            })

            self.channels.push(channel)

            channel.on('signal', (data) => {
                console.log(data)
                cb(null, JSON.stringify(data))
            })

            channel.once('connect', () => {
                const conn = new Connection(toPull.duplex(channel))

                conn.getObservedAddrs = (cb) => cb(null, [])

                cb(true)

                setImmediate(() => {
                    self.listener.emit('connection')
                    handler(null, conn)
                })
            })

            channel.signal(JSON.parse(decoded[1]))

        } else {
            const recipient = new PeerId(decoded[0])

            establishConnection(this.sw, recipient, PROTOCOL_WEBRTC_SIGNALING, (err, conn) => pull(
                pull.once(data),
                lp.encode(),
                conn,
                lp.decode(),
                pull.collect(cb)
            ))
        }
    }),
    lp.encode(),
    conn
)
