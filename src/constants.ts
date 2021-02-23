import Multiaddr from 'multiaddr'

// @ts-ignore
const { name, version } = require('../package.json')

export const NAME = name.replace(/@[a-zA-z0-9\-]+\//, '')

// p2p multi-address code
export const CODE_P2P = Multiaddr.protocols.names['p2p'].code
export const CODE_IP4 = Multiaddr.protocols.names['ip4'].code
export const CODE_IP6 = Multiaddr.protocols.names['ip6'].code
export const CODE_CIRCUIT = Multiaddr.protocols.names['p2p-circuit'].code
export const CODE_TCP = Multiaddr.protocols.names['tcp'].code


// Time to wait for a connection to close gracefully before destroying it manually
export const CLOSE_TIMEOUT = 6000
export const RELAY_CIRCUIT_TIMEOUT = 6000

export const USE_WEBRTC = true
export const WEBRTC_TRAFFIC_PREFIX = 1
export const REMAINING_TRAFFIC_PREFIX = 0
export const WEBRTC_TIMEOUT = 2400

export const OK = new TextEncoder().encode('OK')
export const FAIL = new TextEncoder().encode('FAIL')
export const FAIL_COULD_NOT_REACH_COUNTERPARTY = new TextEncoder().encode('FAIL_COULD_NOT_REACH_COUNTERPARTY')
export const FAIL_COULD_NOT_IDENTIFY_PEER = new TextEncoder().encode('FAIL_COULD_NOT_IDENTIFY_INITIATOR')
export const FAIL_LOOPBACKS_ARE_NOT_ALLOWED = new TextEncoder().encode('FAIL_LOOPBACKS_ARE_NOT_ALLOWED')
export const FAIL_INVALID_PUBLIC_KEY = new TextEncoder().encode('FAIL_INVALID_PUBLIC_KEY')

export const MIGRATE = new TextEncoder().encode('MIGRATE')
export const STOP = new TextEncoder().encode('STOP')
export const RESTART = new TextEncoder().encode('RESTART')
export const PING = new TextEncoder().encode('PING')
export const PONG = new TextEncoder().encode('PONG')

export const RELAY_PAYLOAD_PREFIX = new Uint8Array([0])
export const RELAY_STATUS_PREFIX = new Uint8Array([1])
export const RELAY_WEBRTC_PREFIX = new Uint8Array([2])
export const RELAY_CONNECTION_STATUS_PREFIX = new Uint8Array([3])
export const VALID_PREFIXES = [
  ...RELAY_PAYLOAD_PREFIX,
  ...RELAY_STATUS_PREFIX,
  ...RELAY_WEBRTC_PREFIX,
  ...RELAY_CONNECTION_STATUS_PREFIX
]

export const RELAY = `/hopr/relay-register/${version}`
export const DELIVERY = `/hopr/delivery-register/${version}`
