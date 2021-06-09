import { Address } from 'libp2p/src/peer-store'
import isIpPrivate from 'private-ip'
import { Multiaddr } from 'multiaddr'

export function isMultiaddrLocal(multiaddr: Multiaddr): boolean {
  try {
    const { address } = multiaddr.nodeAddress()
    return isIpPrivate(address)
  } catch (e: any) {
    return false
  }
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

export function localAddressesFirst(addresses: Address[]): Address[] {
  return [...addresses].sort(addressesLocalFirstCompareFunction)
}

export declare type AddressSorter = (input: Address[]) => Address[]
