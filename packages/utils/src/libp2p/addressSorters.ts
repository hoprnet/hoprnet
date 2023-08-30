import { isPrivateAddress, isLocalhost } from '../network/index.js'
import { type Multiaddr, protocols } from '@multiformats/multiaddr'
import type { NetworkInterfaceInfo } from 'os'

const CODE_IP4 = protocols('ip4').code
const CODE_IP6 = protocols('ip6').code
const CODE_P2P = protocols('p2p').code

/**
 * Checks if given Multiaddr encodes a private address
 * @param multiaddr multiaddr to check
 * @returns true if address is a private ip address
 */
export function isMultiaddrLocal(multiaddr: Multiaddr): boolean {
  const tuples = multiaddr.tuples() as [code: number, addr: Uint8Array][]

  let ipFamily: NetworkInterfaceInfo['family']
  switch (tuples[0][0]) {
    case CODE_P2P:
      return false
    case CODE_IP4:
      ipFamily = 'IPv4'
      break
    case CODE_IP6:
      ipFamily = 'IPv6'
      break
    default:
      throw Error(`invalid input arguments`)
  }

  return isLocalhost(tuples[0][1], ipFamily) || isPrivateAddress(tuples[0][1], ipFamily)
}