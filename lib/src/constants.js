"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.PROTOCOL_FORWARD = exports.PROTOCOL_WEBRTC_TURN = exports.PROTOCOL_WEBRTC_TURN_REQUEST = exports.PROTOCOL_HEARTBEAT = exports.PROTOCOL_STUN = exports.PROTOCOL_SETTLE_CHANNEL = exports.PROTOCOL_ONCHAIN_KEY = exports.PROTOCOL_DELIVER_PUBKEY = exports.PROTOCOL_PAYMENT_CHANNEL = exports.PROTOCOL_CRAWLING = exports.PROTOCOL_ACKNOWLEDGEMENT = exports.PROTOCOL_STRING = exports.NAME = exports.MARSHALLED_PUBLIC_KEY_SIZE = exports.MAX_HOPS = exports.PACKET_SIZE = exports.CRAWLING_RESPONSE_NODES = void 0;
exports.CRAWLING_RESPONSE_NODES = 10;
// export const RELAY_FEE = toWei('100', 'wei')
exports.PACKET_SIZE = 500;
exports.MAX_HOPS = 3;
exports.MARSHALLED_PUBLIC_KEY_SIZE = 37;
exports.NAME = 'ipfs'; // 'hopr'
const VERSION = '0.0.1';
const BASESTRING = `/${exports.NAME}/${VERSION}`;
exports.PROTOCOL_STRING = `${BASESTRING}/msg`;
exports.PROTOCOL_ACKNOWLEDGEMENT = `${BASESTRING}/ack`;
exports.PROTOCOL_CRAWLING = `${BASESTRING}/crawl`;
exports.PROTOCOL_PAYMENT_CHANNEL = `${BASESTRING}/payment/open`;
exports.PROTOCOL_DELIVER_PUBKEY = `${BASESTRING}/pubKey`;
exports.PROTOCOL_ONCHAIN_KEY = `${BASESTRING}/onChainKey`;
exports.PROTOCOL_SETTLE_CHANNEL = `${BASESTRING}/payment/settle`;
exports.PROTOCOL_STUN = `${BASESTRING}/stun`;
exports.PROTOCOL_HEARTBEAT = `${BASESTRING}/heartbeat`;
exports.PROTOCOL_WEBRTC_TURN_REQUEST = `${BASESTRING}/webrtc_turn_request`;
exports.PROTOCOL_WEBRTC_TURN = `${BASESTRING}/webrtc_turn`;
exports.PROTOCOL_FORWARD = `${BASESTRING}/forward`;
//# sourceMappingURL=constants.js.map