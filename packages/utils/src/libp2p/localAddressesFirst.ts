import { Address } from 'libp2p/src/peer-store'
import isPrivate from 'libp2p-utils/src/multiaddr/is-private'

function addressesLocalFirstCompareFunction (a: Address, b: Address) {
    const isAPrivate = isPrivate(a.multiaddr)
    const isBPrivate = isPrivate(b.multiaddr)
  
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
  
export function localAddressesFirst (addresses: Address[]) {
    return [...addresses].sort(addressesLocalFirstCompareFunction)
  }