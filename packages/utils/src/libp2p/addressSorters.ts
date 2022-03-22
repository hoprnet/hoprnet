import type { Address } from 'libp2p/src/peer-store/address-book'
import { isPrivateAddress, isLocalhost, ipToU8aAddress } from '../network'
import { type Multiaddr } from 'multiaddr'
import type { NetworkInterfaceInfo } from 'os'

/**
 * Checks if given Multiaddr encodes a private address
 * @param multiaddr multiaddr to check
 * @returns true if address is a private ip address
 */
export function isMultiaddrLocal(multiaddr: Multiaddr): boolean {
  if (multiaddr.toString().startsWith(`/p2p/`)) {
    return false
  }

  const { address, family } = multiaddr.nodeAddress()

  let ipFamily: NetworkInterfaceInfo['family']
  switch (family) {
    case 4:
      ipFamily = 'IPv4'
      break
    case 6:
      ipFamily = 'IPv6'
      break
    default:
      return false
  }

  const u8aAddr = ipToU8aAddress(address, ipFamily)
  return isLocalhost(u8aAddr, ipFamily) || isPrivateAddress(u8aAddr, ipFamily)
}

export function getIpv4LocalAddressClass(address: Multiaddr): 'A' | 'B' | 'C' | 'D' | undefined {
  if (isMultiaddrLocal(address)) {
    if (address.toString().startsWith('/ip4/10.')) return 'A'

    if (/\/ip4\/172\.((1[6-9])|(2\d)|(3[0-1]))\./.test(address.toString())) return 'B'

    if (address.toString().startsWith('/ip4/192.168.')) return 'C'

    if (address.toString().startsWith('/ip4/127.0.0.1')) return 'D'
  }

  return undefined
}

/**
 * Compare two multiaddresses based on their class: A class first, B class second, ...
 * Local addresses take precedence over remote addresses.
 * @param a
 * @param b
 */
export function multiaddressCompareByClassFunction(a: Multiaddr, b: Multiaddr) {
  if (isMultiaddrLocal(a) && isMultiaddrLocal(b)) {
    // Sort based on private address class
    const clsA = getIpv4LocalAddressClass(a)
    const clsB = getIpv4LocalAddressClass(b)
    if (clsA == undefined) return 1
    if (clsB == undefined) return -1
    return clsA.localeCompare(clsB)
  } else if (isMultiaddrLocal(a) && !isMultiaddrLocal(b)) {
    return -1 // Local address takes precedence
  } else if (!isMultiaddrLocal(a) && isMultiaddrLocal(b)) {
    return 1 // Local address takes precedence
  } else return 0
}

function addressesLocalFirstCompareFunction(a: Address, b: Address) {
  const isAPrivate = isMultiaddrLocal(a.multiaddr)
  const isBPrivate = isMultiaddrLocal(b.multiaddr)

  if (isAPrivate && !isBPrivate) {
    return -1
  } else if (!isAPrivate && isBPrivate) {
    return 1
  }

  if (a.isCertified && !b.isCertified) {
    return -1
  } else if (!a.isCertified && b.isCertified) {
    return 1
  }

  return 0
}

/**
 * Take an array of addresses and sorts such that private addresses are first
 * @dev used to run Hopr locally
 * @param addresses
 * @returns
 */
export function localAddressesFirst(addresses: Address[]): Address[] {
  return [...addresses].sort(addressesLocalFirstCompareFunction)
}

export declare type AddressSorter = (input: Address[]) => Address[]
