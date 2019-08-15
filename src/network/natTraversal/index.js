'use strict'

const Basev4 = require('./base/udp4')
const Basev6 = require('./base/udp6')

const Connection = require('interface-connection').Connection
const PeerId = require('peer-id')

const Signalling = require('./signalling')

const { PROTOCOL_WEBRTC_TURN } = require('../../constants')

const mixin = Base =>
    class extends Base {
        constructor(opts) {
            super(opts)

            this.node = opts.libp2p

            this.signalling = new Signalling(opts)

            this.node.on('peer:discover', peerInfo => {
                console.log(peerInfo)
            })
        }

        dial(multiaddr, options, cb) {
            if (typeof options === 'function') {
                cb = options
                options = {}
            }

            let connPromise
            if (multiaddr.getPeerId() !== '16Uiu2HAmSyrYVycqBCWcHyNVQS6zYQcdQbwyov1CDijboVRsQS37') {
                connPromise = this.signalling.relay(PeerId.createFromB58String(multiaddr.getPeerId()))
            } else {
                connPromise = super.dial(multiaddr, options)
            }

            if (cb) {
                const result = new Connection()
                connPromise.then(conn => {
                    result.setInnerConn(conn)
                    cb(null, conn)
                })
                return result
            } else {
                return connPromise
            }
        }

        createListener(options, connHandler) {
            // Creates a UDP listener listening for incoming WebRTC signalling messages
            const listener = super.createListener(options, connHandler)

            this.node.handle(PROTOCOL_WEBRTC_TURN, (err, conn) => this.signalling.handleRequest(err, conn, connHandler))

            if (this.node.bootstrapServers) {
            //    this.signalling.
            }

            return listener
        }

        // dial(multiaddr, options, cb) {
        //     // ==== only for testing ==============

        //     const conn = super.dial(multiaddr, options, err => {
        //         if (err) {
        //         }
        //     })
        // }
    }

module.exports.WebRTCv4 = class WebRTCv4 extends mixin(Basev4) {}

module.exports.WebRTCv6 = class WebRTCv6 extends mixin(Basev6) {}
