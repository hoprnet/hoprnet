import { Address } from 'libp2p/src/peer-store'
import { isPrivateAddress, isLocalhost, ipToU8aAddress } from '../network'
import { Multiaddr } from 'multiaddr'
import type { NetworkInterfaceInfo } from 'os'

/**
 * Checks if given Multiaddr encodes a private address
 * @param multiaddr multiaddr to check
 * @returns true if address is a private ip address
 */
export function isMultiaddrLocal(multiaddr: Multiaddr): boolean {
  try {
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
        throw Error(`Invalid address family in Multiaddr. Got ${family} but expected either '4' or '6'.`)
    }

    const u8aAddr = ipToU8aAddress(address, ipFamily)
    return isLocalhost(u8aAddr, ipFamily) || isPrivateAddress(u8aAddr, ipFamily)
  } catch (e: any) {
    return false
  }
}

export function getIpv4LocalAddressClass(address: Multiaddr): 'A' | 'B' | 'C' | 'D' | undefined {
  if (isMultiaddrLocal(address)) {
    if (address.toString().startsWith('/ip4/10.')) return 'A'

    if (/\/ip4\/172\.((1[6-9])|(2\d)|(3[0-1]))\./.test(address.toString()))
      return 'B'

    if (address.toString().startsWith('/ip4/192.168.')) return 'C'

    if (address.toString().startsWith('/ip4/127.0.0.1')) return 'D'
  }

  return undefined
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
