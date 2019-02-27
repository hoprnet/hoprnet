'use strict'

const SimplePeer = require('simple-peer')
const rlp = require('rlp')
const toPull = require('stream-to-pull-stream')
const { establishConnection, log } = require('../../utils')
const { PROTOCOL_WEBRTC_SIGNALING } = require('../../constants')
const PeerId = require('peer-id')
const paramap = require('pull-paramap')
const pull = require('pull-stream')
const lp = require('pull-length-prefixed')
const wrtc = require('wrtc')
const Connection = require('interface-connection').Connection

module.exports = (self) => (protocol, conn) => pull(
    conn,
    lp.decode(),
    paramap((data, cb) => {
        const decoded = rlp.decode(data)
        if (decoded.length < 2)
            return cb()

        if (self.sw._peerInfo.id.isEqual(decoded[0])) {
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
                trickle: true,
                // allowHalfTrickle: false,
                wrtc: wrtc,
            })

            self.channels.push(channel)

            channel.on('error', (err) => setImmediate(self.listener.emit, 'error', err))
            channel.on('close', () => setImmediate(self.listener.emit, 'close'))

            channel.on('signal', (data) => {
                console.log(data)
                cb(null, Buffer.from(JSON.stringify(data)))
            })

            channel.once('connect', () => {
                const conn = new Connection(toPull.duplex(channel))

                conn.getObservedAddrs = (cb) => cb(null, [])
                setImmediate(self.listener.emit, 'connection', null, conn)

                cb(true)
            })

            channel.signal(JSON.parse(decoded[1]))

        } else {
            const recipient = new PeerId(decoded[0])
            log(self.sw._peerInfo.id, `Relaying signaling message to ${recipient.toB58String()}.`)

            establishConnection(self.sw, recipient, {
                protocol: PROTOCOL_WEBRTC_SIGNALING,
                // big TODO!!!
                peerRouting: self.peerRouting
            }, (err, conn) => {
                if (err) {
                    log(self.sw._peerInfo.id, `Error ${err.message} while relaying signaling message ${JSON.parse(decoded[1])} to ${recipient.toB58String()}.`)
                    return
                }

                pull(
                    pull.once(data),
                    lp.encode(),
                    conn,
                    lp.decode(),
                    pull.drain((data) => {
                        cb(null, data)
                    }, (err) => {
                        cb(err || true)
                    })
                )
            })
        }
    }),
    lp.encode(),
    conn
)
