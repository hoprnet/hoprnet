'use strict'

const Basev4 = require('./base/udp4')
const Basev6 = require('./base/udp6')

const Signalling = require('./signalling')

const { PROTOCOL_WEBRTC_TURN } = require('../../constants')

const mixin = Base =>
    class extends Base {
        constructor(opts) {
            super(opts)

            this.node = opts.libp2p

            this.signalling = new Signalling(opts)
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
