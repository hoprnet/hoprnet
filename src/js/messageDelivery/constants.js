'use strict'

module.exports.PACKET_SIZE = 500

module.exports.MAX_HOPS = 2

module.exports.PROTOCOL_VERSION = '0.0.1'

module.exports.PROTOCOL_NAME = 'ipfs'
module.exports.PROTOCOL_STRING = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_ACKNOWLEDGEMENT = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/acknowledgement/').concat(this.PROTOCOL_VERSION)

module.exports.RELAY_FEE = '100' // Wei