import type { Multiaddr } from 'multiaddr'
import { CODE_IP4, CODE_IP6, CODE_P2P, CODE_CIRCUIT, CODE_TCP } from '../constants'
import { decode } from 'multihashes'
import type { NetworkInterfaceInfo } from 'os'
import { u8aEquals, u8aToNumber } from '@hoprnet/hopr-utils'
import Debug from 'debug'

const MULTIHASH_LENGTH = 37
const MULTIHASH_TYPE = 'identity'

type DirectAddress = {
  type: NetworkInterfaceInfo['family']
  address: Uint8Array
  port: number
  node?: Uint8Array
}

type CircuitAddress = {
  type: 'p2p'
  relayer: Uint8Array
  node: Uint8Array
}

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
 * Checks a given Multihash
 * @param mh Multihash to check
 */
function parseMultihash(mh: Uint8Array): { valid: false } | { valid: true; result: ReturnType<typeof decode> } {
  let decoded: ReturnType<typeof decode>
  try {
    decoded = decode(mh)
  } catch (err) {
    log(`address is not a valid Multihash`, err)
    return { valid: false }
  }

  if (decoded.name !== MULTIHASH_TYPE || decoded.length != MULTIHASH_LENGTH) {
    log(`address length is not ${MULTIHASH_LENGTH} bytes long or type is not ${MULTIHASH_TYPE}`)
    return { valid: false }
  }

  return {
    valid: true,
    result: decoded
  }
}

/**
 * Checks a given circuit address
 * @param maTuples tuples of a Multiaddr
 */
function parseCircuitAddress(maTuples: [code: number, addr: Uint8Array][]): ParseResult<CircuitAddress> {
  if (
    maTuples.length != 3 ||
    maTuples[0].length < 2 ||
    maTuples[0][0] != CODE_P2P ||
    maTuples[1].length < 1 ||
    maTuples[1][0] != CODE_CIRCUIT ||
    maTuples[2].length < 2 ||
    maTuples[2][0] != CODE_P2P
  ) {
    return { valid: false }
  }

  // first address and second address WITHOUT length prefix
  const pubKeys = [maTuples[0][1].slice(1), maTuples[2][1].slice(1)]

  const decoded = []
  for (const pubKey of pubKeys) {
    let tmp = parseMultihash(pubKey)

    if (!tmp.valid) {
      return { valid: false }
    }

    decoded.push(tmp.result.digest)
  }

  if (u8aEquals(decoded[0], decoded[1])) {
    log(`first and second address must not be the same`)
    return { valid: false }
  }

  return {
    valid: true,
    address: {
      type: 'p2p',
      relayer: decoded[0],
      node: decoded[1]
    }
  }
}

/**
 * Checks a given direct address
 * @param maTuples tuples of a Multiaddr
 */
function parseDirectAddress(maTuples: [code: number, addr: Uint8Array][]): ParseResult<DirectAddress> {
  if (
    maTuples.length < 2 ||
    maTuples[0].length < 2 ||
    ![CODE_IP4, CODE_IP6].includes(maTuples[0][0]) ||
    maTuples[1].length < 2 ||
    maTuples[1][0] != CODE_TCP ||
    maTuples[1][1].length == 0
  ) {
    return { valid: false }
  }

  let family: NetworkInterfaceInfo['family']

  switch (maTuples[0][0]) {
    case CODE_IP4:
      family = 'IPv4'
      break
    case CODE_IP6:
      family = 'IPv6'
      break
    default:
      return { valid: false }
  }

  let result: DirectAddress = {
    port: u8aToNumber(maTuples[1][1]),
    address: maTuples[0][1],
    type: family
  }


  if (maTuples.length == 3) {
    if (maTuples[2].length < 2 || maTuples[2][0] != CODE_P2P || maTuples[2][1].length == 0) {
      return { valid: false }
    }

    const decoded = parseMultihash(maTuples[2][1].slice(1))

    if (!decoded.valid) {
      return { valid: false }
    }

    result.node = decoded.result.digest
  }

  return { valid: true, address: result }
}

/**
 * Checks and parses a given Multiaddr
 * @param ma Multiaddr to check and parse
 */
export function parseAddress(ma: Multiaddr): ParseResult<DirectAddress | CircuitAddress> {
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
