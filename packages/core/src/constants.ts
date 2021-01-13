import { durations } from '@hoprnet/hopr-utils'
import BN from 'bn.js'

export const TICKET_AMOUNT = 1000000000000000 // 0.001 HOPR
export const TICKET_WIN_PROB = 1 // 100%
export const PACKET_SIZE = 500
export const MARSHALLED_PUBLIC_KEY_SIZE = 37
export const NAME = 'ipfs' // 'hopr'

export const FULL_VERSION = require('../package.json').version
const packageVersion = FULL_VERSION.split('.')
export const VERSION = packageVersion[0] + '.' + packageVersion[1] + '.0' // Version on major versions only
const PROTOCOL_NAME = 'hopr'

export const PROTOCOL_STRING = `/${PROTOCOL_NAME}/msg/${VERSION}`
export const PROTOCOL_ACKNOWLEDGEMENT = `/${PROTOCOL_NAME}/ack/${VERSION}`
export const PROTOCOL_PAYMENT_CHANNEL = `/${PROTOCOL_NAME}/payment/open/${VERSION}`
export const PROTOCOL_ONCHAIN_KEY = `/${PROTOCOL_NAME}/onChainKey/${VERSION}`
export const PROTOCOL_HEARTBEAT = `/${PROTOCOL_NAME}/heartbeat/${VERSION}`
export const DEFAULT_STUN_PORT = 3478

export const HEARTBEAT_REFRESH = 103000
export const HEARTBEAT_INTERVAL = 50000
export const HEARTBEAT_INTERVAL_VARIANCE = 5000

export const MAX_PARALLEL_CONNECTIONS = 5

export const HEARTBEAT_TIMEOUT = durations.seconds(3)

export const MAX_PACKET_DELAY = 200

export const MAX_HOPS = 2
export const PATH_RANDOMNESS = 0.1
export const MAX_PATH_ITERATIONS = 100
export const NETWORK_QUALITY_THRESHOLD = 0.5
export const MINIMUM_REASONABLE_CHANNEL_STAKE = new BN(TICKET_AMOUNT).muln(10)
export const MAX_NEW_CHANNELS_PER_TICK = 10

export const MIN_NATIVE_BALANCE = 1000

export const CHECK_TIMEOUT = 30000
