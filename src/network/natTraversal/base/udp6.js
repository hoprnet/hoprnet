'use strict'

const WebRTC = require('./webrtc')

module.exports = class WebRTCv6 extends WebRTC {
    constructor(opts) {
        super(opts)

        this.tag = 'WebRTCv6'
        this.socketType = 'udp6'
        this.family = 'ipv6'
    }
}