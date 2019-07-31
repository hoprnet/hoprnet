'use strict'

const WebRTC = require('./webrtc')

module.exports = class WebRTCv4 extends WebRTC {
    constructor(opts) {
        super(opts)

        this.tag = 'WebRTCv4'
        this.socketType = 'udp4'
        this.family = 'ipv4'
    }
}
