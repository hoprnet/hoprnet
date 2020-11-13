import { durations } from '@hoprnet/hopr-utils'

export const CRAWLING_RESPONSE_NODES = 10
export const RELAY_FEE = 10000000000000 // 0.00001 HOPR
export const TICKET_AMOUNT = 1000000000000000 // 0.001 HOPR
export const TICKET_WIN_PROB = 1 // 100%
export const PACKET_SIZE = 500
export const MAX_HOPS = 3
export const MARSHALLED_PUBLIC_KEY_SIZE = 37
export const NAME = 'ipfs' // 'hopr'

const packageVersion = require('../package.json').version.split('.')
export const VERSION = packageVersion[0] + '.' + packageVersion[0] + '.0' // Version on major versions only
const PROTOCOL_NAME = 'hopr'

export const PROTOCOL_STRING = `/${PROTOCOL_NAME}/msg/${VERSION}`
export const PROTOCOL_ACKNOWLEDGEMENT = `/${PROTOCOL_NAME}/ack/${VERSION}`
export const PROTOCOL_CRAWLING = `/${PROTOCOL_NAME}/crawl/${VERSION}`
export const PROTOCOL_PAYMENT_CHANNEL = `/${PROTOCOL_NAME}/payment/open/${VERSION}`
export const PROTOCOL_ONCHAIN_KEY = `/${PROTOCOL_NAME}/onChainKey/${VERSION}`
export const PROTOCOL_HEARTBEAT = `/${PROTOCOL_NAME}/heartbeat/${VERSION}`
export const DEFAULT_STUN_PORT = 3478

export const HEARTBEAT_REFRESH_TIME = 103 * 1000
export const HEARTBEAT_INTERVAL_LOWER_BOUND = 41 * 1000
export const HEARTBEAT_INTERVAL_UPPER_BOUND = 59 * 1000

export const MAX_PARALLEL_CONNECTIONS = 10

export const BLACKLIST_TIMEOUT = durations.seconds(47)
export const HEARTBEAT_TIMEOUT = durations.seconds(3)

export const CRAWL_TIMEOUT = 100_000 // ~15 mins

export const MAX_PACKET_DELAY = 200
