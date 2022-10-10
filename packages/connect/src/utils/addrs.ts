import type { Multiaddr } from '@multiformats/multiaddr'
import { CODE_IP4, CODE_IP6, CODE_P2P, CODE_CIRCUIT, CODE_TCP } from '../constants.js'
// @ts-ignore untyped library
import { decode, Digest } from 'multiformats/hashes/digest'
// @ts-ignore untyped library
import { identity } from 'multiformats/hashes/identity'

import { u8aToNumber, u8aCompare } from '@hoprnet/hopr-utils'
import Debug from 'debug'

const MULTIHASH_LENGTH = 37
const MULTIHASH_TYPE = 'identity'

export enum AddressType {
  IPv4 = 'IPv4',
  IPv6 = 'IPv6',
  P2P = 'p2p'
}

export type DirectAddress = {
  type: AddressType.IPv4 | AddressType.IPv6
  address: Uint8Array
  port: number
}

export type CircuitAddress = {
  type: AddressType.P2P
  relayer: Uint8Array
}

export type ValidAddress = DirectAddress | CircuitAddress

type ParseResult<Type> =
  | {
      valid: true
      address: Type
    }
  | {
      valid: false
    }

const log = Debug('hopr-connect:addr')

/**
 * Checks and parses a given Multihash
 * @param mh Multihash to check
 */
function parseMultihash(mh: Uint8Array): { valid: false } | { valid: true; result: Digest } {
  let decoded: Digest
  try {
    decoded = decode(mh)
  } catch (err) {
    console.log(err)
    log(`address is not a valid Multihash`, err)
    return { valid: false }
  }

  if (decoded.code != identity.code || decoded.size != MULTIHASH_LENGTH) {
    log(`address length is not ${MULTIHASH_LENGTH} bytes long or type is not ${MULTIHASH_TYPE}`)
    return { valid: false }
  }

  return {
    valid: true,
    result: decoded
  }
}

/**
 * Checks and parses a given circuit address
 * @param maTuples tuples of a Multiaddr
 */
function parseCircuitAddress(maTuples: [code: number, addr: Uint8Array][]): ParseResult<CircuitAddress> {
  if (
    maTuples.length < 2 ||
    maTuples[0].length < 2 ||
    maTuples[0][0] != CODE_P2P ||
    maTuples[1].length < 1 ||
    maTuples[1][0] != CODE_CIRCUIT
  ) {
    return { valid: false }
  }

  // relayer address without length prefix
  const relayerRaw = maTuples[0][1].slice(1)

  let tmp = parseMultihash(relayerRaw)

  if (!tmp.valid) {
    return { valid: false }
  }

  const relayer = tmp.result.digest

  return {
    valid: true,
    address: {
      type: AddressType.P2P,
      relayer
    }
  }
}

/**
 * Checks and parses a given direct TCP address
 * @param maTuples tuples of a Multiaddr
 */
function parseDirectAddress(maTuples: [code: number, addr: Uint8Array][]): ParseResult<DirectAddress> {
  if (
    maTuples.length < 2 ||
    maTuples[0].length < 2 ||
    ![CODE_IP4, CODE_IP6].includes(maTuples[0][0]) ||
    maTuples[1].length < 2 ||
    maTuples[1][0] != CODE_TCP || // TODO: We should also consider UDP a valid direct address here
    maTuples[1][1].length == 0
  ) {
    return { valid: false }
  }

  let family: AddressType.IPv4 | AddressType.IPv6

  switch (maTuples[0][0]) {
    case CODE_IP4:
      family = AddressType.IPv4
      break
    case CODE_IP6:
      family = AddressType.IPv6
      break
    default:
      return { valid: false }
  }

  let result: DirectAddress = {
    port: u8aToNumber(maTuples[1][1]),
    address: maTuples[0][1],
    type: family
  }

  return { valid: true, address: result }
}

/**
 * Checks and parses a given Multiaddr
 * @param ma Multiaddr to check and parse
 */
export function parseAddress(ma: Multiaddr): ParseResult<ValidAddress> {
  const tuples = ma.tuples() as [code: number, addr: Uint8Array][]

  switch (tuples[0][0]) {
    case CODE_IP4:
    case CODE_IP6:
      return parseDirectAddress(tuples)
    case CODE_P2P:
      return parseCircuitAddress(tuples)
    default:
      return { valid: false }
  }
}

/**
 * Compares the IP addresses and destination ports of two direct multi-addresses (non-circuit ones).
 * @param a Multiaddr 1
 * @param b Multiaddr 2
 */
export function compareDirectConnectionInfo(a: Multiaddr, b: Multiaddr): boolean {
  const va1 = parseAddress(a)
  const va2 = parseAddress(b)

  if (!va1.valid || !va2.valid) {
    return false
  }

  if (va1.address.type !== va2.address.type) {
    return false
  }

  if (va1.address.type === AddressType.P2P) {
    return false
  }

  if (va1.address.type === va2.address.type) {
    return u8aCompare(va1.address.address, va2.address.address) == 0 && va1.address.port == va2.address.port
  }

  // TODO: We should also compare protocol (TCP/UDP)

  return false
}
