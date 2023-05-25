import { protocols } from '@multiformats/multiaddr'
import { pickVersion } from '@hoprnet/hopr-utils'

import type { Network } from './types.js'

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

// In order to traverse NATs, nodes needs to maintain
// a connection to at least one of the entry nodes.
// To always have a fallback option, nodes connect to multiple
// entry nodes up to MAX_RELAYS_PER_NODE. If the number of active
// connections falls below MIN_RELAYS_PER_NODE, the node will
// actively try to connect to different entry nodes
export const MAX_RELAYS_PER_NODE = 5
export const MIN_RELAYS_PER_NODE = 3

/**
 * @param network [optional] isolate from nodes running in other networks
 * @returns the relay request protocol strings
 */
export function CAN_RELAY_PROTOCOLS(network?: string, networks?: Network[]): string[] {
  return determine_protocols('can-relay', network, networks)
}

/**
 * @param network [optional] isolate from nodes running in other networks
 * @returns the relay request protocol strings
 */
export function RELAY_PROTOCOLS(network?: string, networks?: Network[]): string[] {
  return determine_protocols('relay', network, networks)
}

/**
 * @param network [optional] isolate from nodes running in other networks
 * @returns the relay delivery protocol strings
 */
export function DELIVERY_PROTOCOLS(network?: string, networks?: Network[]): string[] {
  return determine_protocols('delivery', network, networks)
}

/**
 * @param tag protocol tag which should be used
 * @param network [optional] isolate from nodes running in other networks
 * @param networks [optional] supported networks which can be considered
 * @returns the supported protocol strings
 *
 * This function uses the given networks information to determine the
 * supported protocols. If no network is given, it will return a list with a
 * single, version-specific entry, e.g.:
 *
 *   /hopr-connect/{TAG}/1.90
 *
 * When an network is given, multiple protocols are returned. To illustrate
 * this the network 'monte_rosa' and releases 'paleochora', 'valencia' , 'bogota' and 'riga'
 * are used here:
 *
 *   /hopr-connect/monte_rosa/{TAG}/1.89
 *   /hopr-connect/monte_rosa/{TAG}/1.90
 *   /hopr-connect/monte_rosa/{TAG}/1.91
 *   /hopr-connect/monte_rosa/{TAG}/1.92
 */
function determine_protocols(tag: string, network?: string, networks?: Network[]): string[] {
  const supportedNetworkNames = networks?.map((env) => env.id)
  let protos: string[] = []

  // only add network-specific protocols if we run a supported network
  if (network && supportedNetworkNames && supportedNetworkNames.indexOf(network) > -1) {
    const env = networks?.find((el) => el.id === network)
    if (env) {
      const versions = env.versionRange.split('||')
      versions.forEach((v: string) => {
        let proto
        if (v === '') {
          proto = ''
        }
        if (v === '*') {
          // the placeholder '*' will open up the protocol to the entire
          // network, otherwise we pin to the given version
          proto = `/${NAME}/${network}/${tag}`
        } else {
          // pinning each versions allows to support other protocol versions
          // within the same network
          proto = `/${NAME}/${network}/${tag}/${pickVersion(v)}`
        }

        if (proto != '' && protos.indexOf(proto) == -1) {
          protos.push(proto)
        }
      })
    }
  } else {
    // legacy entry which can also be used for internal testing
    protos.push(`/${NAME}/${tag}/${NORMALIZED_VERSION}`)
  }

  return protos
}
