import { pickVersion } from '@hoprnet/hopr-utils'

// Don't do typechecks on JSON files
// @ts-ignore
import pkg from '../package.json' assert { type: 'json' }

export const PACKET_SIZE = 500
export const FULL_VERSION = pkg.version

export const VERSION = pickVersion(pkg.version)

export const INTERMEDIATE_HOPS = 3 // require 3 intermediary nodes
export const PATH_RANDOMNESS = 0.1
export const MAX_PATH_ITERATIONS = 100
export const NETWORK_QUALITY_THRESHOLD = 0.5
export const MAX_NEW_CHANNELS_PER_TICK = 5
export const MAX_HOPS = 3 // 3 intermediate hops and one recipient

export const MAX_PARALLEL_PINGS = 14

export const CHECK_TIMEOUT = 60000
export const ACKNOWLEDGEMENT_TIMEOUT = 2000

export const PEER_METADATA_PROTOCOL_VERSION = 'protocol_version'
