import { protocols } from '@multiformats/multiaddr'
import { pickVersion } from '@hoprnet/hopr-utils'
// Do not type-check JSON files
// @ts-ignore
import pkg from '../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

// Use name without organisation prefix
export const NAME = pkg.name.replace(/@[a-zA-z0-9\-]+\//, '')

// p2p multi-address code
export const CODE_P2P = protocols('p2p').code
export const CODE_IP4 = protocols('ip4').code
export const CODE_IP6 = protocols('ip6').code
export const CODE_DNS4 = protocols('dns4').code
export const CODE_DNS6 = protocols('dns6').code

export const CODE_CIRCUIT = protocols('p2p-circuit').code
export const CODE_TCP = protocols('tcp').code
export const CODE_UDP = protocols('udp').code

// Time to wait for a connection to close gracefully before destroying it manually
export const CLOSE_TIMEOUT = 6000 // ms
export const RELAY_CIRCUIT_TIMEOUT = 6000 // ms
export const RELAY_CONTACT_TIMEOUT = 3000 // ms

export const WEBRTC_TIMEOUT = 2400 // ms

// Keys in the DHT have a TTL of 24 hours, hence
// relay keys need to be reset on daily base.
// Interval to renew the DHT entry
export const DEFAULT_DHT_ENTRY_RENEWAL = 8 * 60 * 60 * 1000 // 8 hours

// Use default UTF-8 text encoding
const encoder = new TextEncoder()

export const OK = encoder.encode('OK')
export const FAIL = encoder.encode('FAIL')

export enum StatusMessages {
  PING,
  PONG
}

export enum ConnectionStatusMessages {
  STOP,
  RESTART,
  UPGRADED
}

export enum RelayPrefix {
  PAYLOAD,
  STATUS_MESSAGE,
  CONNECTION_STATUS,
  WEBRTC_SIGNALLING
}

export const MAX_RELAYS_PER_NODE = 5
export const MIN_RELAYS_PER_NODE = 3

/**
 * @param environment [optional] isolate from nodes running in other environments
 * @returns the relay request protocol string
 */
export function CAN_RELAY_PROTCOL(environment?: string): string {
  if (environment) {
    return `/${NAME}/${environment}/can-relay/${NORMALIZED_VERSION}`
  }
  return `/${NAME}/can-relay/${NORMALIZED_VERSION}`
}

/**
 * @param environment [optional] isolate from nodes running in other environments
 * @returns the relay request protocol string
 */
export function RELAY_PROTCOL(environment?: string): string {
  if (environment) {
    return `/${NAME}/${environment}/relay/${NORMALIZED_VERSION}`
  }
  return `/${NAME}/relay/${NORMALIZED_VERSION}`
}

/**
 * @param environment [optional] isolate from nodes running in other environments
 * @returns the relay delivery protocol string
 */
export function DELIVERY_PROTOCOL(environment?: string): string {
  if (environment) {
    return `/${NAME}/${environment}/delivery/${NORMALIZED_VERSION}`
  }
  return `/${NAME}/delivery/${NORMALIZED_VERSION}`
}
