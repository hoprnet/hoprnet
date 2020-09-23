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

export const RELAY_REGISTER = '/hopr/relay-register/0.0.1'
export const DELIVERY_REGISTER = '/hopr/delivery-register/0.0.1'

export const STUN_SERVERS = [
  {
    hostname: 'stun.l.google.com',
    port: 19302,
  },
  {
    hostname: 'stun.1und1.de',
    port: 3478,
  },
]
