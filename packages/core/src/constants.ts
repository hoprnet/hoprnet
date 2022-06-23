import { pickVersion } from '@hoprnet/hopr-utils'

// Don't do typechecks on JSON files
// @ts-ignore
import pkg from '../package.json' assert { type: 'json' }

export const PACKET_SIZE = 500
export const FULL_VERSION = pkg.version

export const VERSION = pickVersion(pkg.version)

// The timeout must include the time necessary to traverse
// NATs which might include several round trips
export const HEARTBEAT_TIMEOUT = 30000
export const HEARTBEAT_INTERVAL = 60000
export const HEARTBEAT_THRESHOLD = 60000
export const HEARTBEAT_INTERVAL_VARIANCE = 2000

export const MAX_PACKET_DELAY = 200

export const INTERMEDIATE_HOPS = 3 // require 3 intermediary nodes
export const PATH_RANDOMNESS = 0.1
export const MAX_PATH_ITERATIONS = 100
export const NETWORK_QUALITY_THRESHOLD = 0.5
export const MAX_NEW_CHANNELS_PER_TICK = 5
export const MAX_HOPS = 3 //3 intermediate hops and one recipient

export const CHECK_TIMEOUT = 60000
export const ACKNOWLEDGEMENT_TIMEOUT = 2000
