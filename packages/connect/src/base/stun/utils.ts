import { Multiaddr } from '@multiformats/multiaddr'

import { u8aAddrToString, u8aToNumber } from '@hoprnet/hopr-utils'

import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from '../../constants.js'

export function parseStunAddress(addr: Multiaddr): { address: string; port: number } {
  const tuples = addr.tuples()

  if (tuples.length == 0) {
    throw Error(`Cannot perform STUN request: empty Multiaddr`)
  }

  let address: string

  switch (tuples[0][0]) {
    case CODE_DNS4:
    case CODE_DNS6:
      address = new TextDecoder().decode(tuples[0][1]?.slice(1) as Uint8Array)
      break
    case CODE_IP6:
      address = u8aAddrToString(tuples[0][1] as Uint8Array, 'IPv6')
      break
    case CODE_IP4:
      address = `::ffff:${u8aAddrToString(tuples[0][1] as Uint8Array, 'IPv4')}`
      break
    default:
      throw Error(`Invalid address: ${addr.toString()}`)
  }

  if (tuples.length < 2) {
    throw Error(`No port given.`)
  }

  const port: number = u8aToNumber(tuples[1][1] as Uint8Array)

  return {
    address,
    port
  }
}
