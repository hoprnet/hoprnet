const pkg = require('../package.json')

export const PACKET_SIZE = 500
export const FULL_VERSION = pkg.version

const packageVersion = pkg.version.split('.')
export const VERSION = packageVersion[0] + '.' + packageVersion[1] + '.0' // Version on major versions only

export const DEFAULT_STUN_PORT = 3478

export const HEARTBEAT_INTERVAL = 3000
export const HEARTBEAT_INTERVAL_VARIANCE = 2000

export const MAX_PARALLEL_CONNECTIONS = 5

export const HEARTBEAT_TIMEOUT = 4000

export const MAX_PACKET_DELAY = 200

export const INTERMEDIATE_HOPS = 3 // require 3 intermediary nodes
export const PATH_RANDOMNESS = 0.1
export const MAX_PATH_ITERATIONS = 100
export const NETWORK_QUALITY_THRESHOLD = 0.5
export const MAX_NEW_CHANNELS_PER_TICK = 5

export const CHECK_TIMEOUT = 60000
