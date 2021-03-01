import Multiaddr from 'multiaddr'

// @ts-ignore
const { name, version } = require('../package.json')

// Use name without organisation prefix
export const NAME = name.replace(/@[a-zA-z0-9\-]+\//, '')

// p2p multi-address code
export const CODE_P2P = Multiaddr.protocols.names['p2p'].code
export const CODE_IP4 = Multiaddr.protocols.names['ip4'].code
export const CODE_IP6 = Multiaddr.protocols.names['ip6'].code
export const CODE_CIRCUIT = Multiaddr.protocols.names['p2p-circuit'].code
export const CODE_TCP = Multiaddr.protocols.names['tcp'].code

// Time to wait for a connection to close gracefully before destroying it manually
export const CLOSE_TIMEOUT = 6000 // ms
export const RELAY_CIRCUIT_TIMEOUT = 6000 // ms
export const RELAY_CONTACT_TIMEOUT = 3000 // ms

// Either set on ALL nodes to true or NONE
// @dev mixed operation is neither tested nor implemented
export const USE_WEBRTC = true
export const WEBRTC_TIMEOUT = 2400 // ms

// Use default UTF-8 text encoding
const encoder = new TextEncoder()

export const OK = encoder.encode('OK')
export const FAIL = encoder.encode('FAIL')
export const FAIL_COULD_NOT_REACH_COUNTERPARTY = encoder.encode('FAIL_COULD_NOT_REACH_COUNTERPARTY')
export const FAIL_COULD_NOT_IDENTIFY_PEER = encoder.encode('FAIL_COULD_NOT_IDENTIFY_INITIATOR')
export const FAIL_LOOPBACKS_ARE_NOT_ALLOWED = encoder.encode('FAIL_LOOPBACKS_ARE_NOT_ALLOWED')
export const FAIL_INVALID_PUBLIC_KEY = encoder.encode('FAIL_INVALID_PUBLIC_KEY')

export const STOP = encoder.encode('STOP')
export const RESTART = encoder.encode('RESTART')
export const PING = encoder.encode('PING')
export const PONG = encoder.encode('PONG')

export const RELAY_PAYLOAD_PREFIX = Uint8Array.from([0])
export const RELAY_STATUS_PREFIX = Uint8Array.from([1])
export const RELAY_WEBRTC_PREFIX = Uint8Array.from([2])
export const RELAY_CONNECTION_STATUS_PREFIX = Uint8Array.from([3])
export const VALID_PREFIXES = [
  ...RELAY_PAYLOAD_PREFIX,
  ...RELAY_STATUS_PREFIX,
  ...RELAY_WEBRTC_PREFIX,
  ...RELAY_CONNECTION_STATUS_PREFIX
]

export const RELAY = `/${NAME}/relay/${version}`
export const DELIVERY = `/${NAME}/delivery/${version}`
