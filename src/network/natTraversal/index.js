'use strict'

const Basev4 = require('./base/udp4')
const Basev6 = require('./base/udp6')

const mixin = Base => class extends Base {

}

module.exports.WebRTCv4 = class WebRTCv4 extends mixin(Basev4) {}

module.exports.WebRTCv6 = class WebRTCv6 extends mixin(Basev6) {}