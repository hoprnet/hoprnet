const pkg = require('../package.json')

export const PACKET_SIZE = 500
export const FULL_VERSION = pkg.version

const packageVersion = pkg.version.split('.')
export const VERSION = packageVersion[0] + '.' + packageVersion[1] + '.0' // Version on major versions only

const PROTOCOL_NAME = 'hopr'

export const PROTOCOL_STRING = `/${PROTOCOL_NAME}/msg/${VERSION}`
export const PROTOCOL_ACKNOWLEDGEMENT = `/${PROTOCOL_NAME}/ack/${VERSION}`
export const PROTOCOL_PAYMENT_CHANNEL = `/${PROTOCOL_NAME}/payment/open/${VERSION}`
export const PROTOCOL_ONCHAIN_KEY = `/${PROTOCOL_NAME}/onChainKey/${VERSION}`
export const PROTOCOL_HEARTBEAT = `/${PROTOCOL_NAME}/heartbeat/${VERSION}`
export const DEFAULT_STUN_PORT = 3478

export const HEARTBEAT_INTERVAL = 3000
export const HEARTBEAT_INTERVAL_VARIANCE = 200

export const MAX_PARALLEL_CONNECTIONS = 5

export const HEARTBEAT_TIMEOUT = 4000

export const MAX_PACKET_DELAY = 200

export const INTERMEDIATE_HOPS = 3 // require 3 intermediary nodes
export const PATH_RANDOMNESS = 0.1
export const MAX_PATH_ITERATIONS = 100
export const NETWORK_QUALITY_THRESHOLD = 0.5
export const MAX_NEW_CHANNELS_PER_TICK = 5

export const CHECK_TIMEOUT = 60000
