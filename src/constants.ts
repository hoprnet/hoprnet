import { version } from '../package.json'

// p2p multi-address code
export const CODE_P2P = 421
export const CODE_CIRCUIT = 290

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
export const STOP = new TextEncoder().encode('STOP')
export const RESTART = new TextEncoder().encode('RESTART')
export const PING = new TextEncoder().encode('PING')
export const PING_RESPONSE = new TextEncoder().encode('PING_RESPONSE')

export const RELAY_PAYLOAD_PREFIX = new Uint8Array([0])
export const RELAY_STATUS_PREFIX = new Uint8Array([1])
export const RELAY_WEBRTC_PREFIX = new Uint8Array([2])

export const RELAY = `/hopr/relay-register/${version}`
export const DELIVERY = `/hopr/delivery-register/${version}`
