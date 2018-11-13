'use strict'

module.exports.PACKET_SIZE = 500

module.exports.MAX_HOPS = 3

module.exports.PROTOCOL_VERSION = '0.0.1'

module.exports.PROTOCOL_NAME = 'ipfs'
module.exports.PROTOCOL_STRING = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/msg').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_ACKNOWLEDGEMENT = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/acknowledgement/').concat(this.PROTOCOL_VERSION)
module.exports.PROTOCOL_CRAWLING = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/crawl').concat(this.PROTOCOL_VERSION)



module.exports.PROTOCOL_DELIVER_PUBKEY = '/'.concat(this.PROTOCOL_NAME.toLowerCase()).concat('/pubKey').concat(this.PROTOCOL_VERSION)



module.exports.MARSHALLED_PUBLIC_KEY_SIZE = 37

module.exports.CRAWLING_RESPONSE_NODES = 10

module.exports.RELAY_FEE = '100' // Wei

module.exports.DEMO = false