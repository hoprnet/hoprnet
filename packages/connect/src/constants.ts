import Multiaddr from 'multiaddr'

const { name, version } = require('../package.json')

// Use name without organisation prefix
export const NAME = name.replace(/@[a-zA-z0-9\-]+\//, '')

// p2p multi-address code
export const CODE_P2P = Multiaddr.protocols.names['p2p'].code
export const CODE_IP4 = Multiaddr.protocols.names['ip4'].code
export const CODE_IP6 = Multiaddr.protocols.names['ip6'].code
export const CODE_DNS4 = Multiaddr.protocols.names['dns4'].code
export const CODE_DNS6 = Multiaddr.protocols.names['dns6'].code

export const CODE_CIRCUIT = Multiaddr.protocols.names['p2p-circuit'].code
export const CODE_TCP = Multiaddr.protocols.names['tcp'].code
export const CODE_UDP = Multiaddr.protocols.names['udp'].code

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

export function isValidPrefix(prefix: RelayPrefix): boolean {
  switch (prefix) {
    case RelayPrefix.PAYLOAD:
    case RelayPrefix.STATUS_MESSAGE:
    case RelayPrefix.CONNECTION_STATUS:
    case RelayPrefix.WEBRTC_SIGNALLING:
      return true
    default:
      return false
  }
}

export const MAX_RELAYS_PER_NODE = 5

/**
 * @param environment [optional] isolate from nodes running in other environments
 * @returns the relay request protocol string
 */
export function CAN_RELAY_PROTCOL(environment?: string): string {
  if (environment) {
    return `/${NAME}/${environment}/can-relay/${version}`
  }
  return `/${NAME}/can-relay/${version}`
}

/**
 * @param environment [optional] isolate from nodes running in other environments
 * @returns the relay request protocol string
 */
export function RELAY_PROTCOL(environment?: string): string {
  if (environment) {
    return `/${NAME}/${environment}/relay/${version}`
  }
  return `/${NAME}/relay/${version}`
}

/**
 * @param environment [optional] isolate from nodes running in other environments
 * @returns the relay delivery protocol string
 */
export function DELIVERY_PROTOCOL(environment?: string): string {
  if (environment) {
    return `/${NAME}/${environment}/delivery/${version}`
  }
  return `/${NAME}/delivery/${version}`
}
