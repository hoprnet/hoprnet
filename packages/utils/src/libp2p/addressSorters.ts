import {
  isPrivateAddress,
  isLocalhost,
  inSameNetwork,
  PRIVATE_V4_CLASS_A,
  PRIVATE_V4_CLASS_B,
  PRIVATE_V4_CLASS_C,
  CARRIER_GRADE_NAT_NETWORK,
  isReservedAddress,
  isLinkLocaleAddress
} from '../network/index.js'

import { type Multiaddr, protocols } from '@multiformats/multiaddr'
import type { NetworkInterfaceInfo } from 'os'

const CODE_IP4 = protocols('ip4').code
const CODE_IP6 = protocols('ip6').code
const CODE_P2P = protocols('p2p').code
const CODE_DNS4 = protocols('dns4').code

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
      return isLocalhost(tuples[0][1], ipFamily) || isPrivateAddress(tuples[0][1], ipFamily)
    case CODE_IP6:
      ipFamily = 'IPv6'
      return isLocalhost(tuples[0][1], ipFamily) || isPrivateAddress(tuples[0][1], ipFamily)
    case CODE_DNS4:
      const strTuples = multiaddr.stringTuples() as [code: number, addr: string][]
      return strTuples[0][1] == 'localhost'
    default:
      throw Error(`invalid input arguments`)
  }

  return isLocalhost(tuples[0][1], ipFamily) || isPrivateAddress(tuples[0][1], ipFamily)
}

export enum AddressClass {
  Public,
  Public6,
  Circuit,
  PrivateA,
  PrivateB,
  PrivateC,
  CarrierNAT,
  Loopback,
  Loopback6,
  Invalid,
  Invalid6
}

function addressPriorityPublic(addrClass: AddressClass) {
  switch (addrClass) {
    case AddressClass.Public:
      return 0
    case AddressClass.Public6:
      return 1
    case AddressClass.CarrierNAT:
      return 2
    case AddressClass.Circuit:
      return 3
    case AddressClass.PrivateA:
      return 4
    case AddressClass.PrivateB:
      return 5
    case AddressClass.PrivateC:
      return 6
    case AddressClass.Loopback:
      return 7
    case AddressClass.Loopback6:
      return 8
    case AddressClass.Invalid:
      return 9
    case AddressClass.Invalid6:
      return 10
  }
}

function addressPriorityLocal(addrClass: AddressClass) {
  switch (addrClass) {
    case AddressClass.Loopback:
      return 0
    case AddressClass.Loopback6:
      return 1
    case AddressClass.PrivateA:
      return 2
    case AddressClass.PrivateB:
      return 3
    case AddressClass.PrivateC:
      return 4
    case AddressClass.Circuit:
      return 5
    case AddressClass.Public:
      return 6
    case AddressClass.Public6:
      return 7
    case AddressClass.CarrierNAT:
      return 8
    case AddressClass.Invalid:
      return 9
    case AddressClass.Invalid6:
      return 10
  }
}

/**
 * Map a given Multiaddr to a network class, e.g. `Public` address
 * @param ma Multiaddr to determine class
 * @returns the assigned class
 */
export function maToClass(ma: Multiaddr): AddressClass {
  const tuples = ma.tuples() as [code: number, addr: Uint8Array][]

  switch (tuples[0][0]) {
    case CODE_P2P:
      return AddressClass.Circuit
    case CODE_IP4:
      if (inSameNetwork(tuples[0][1], PRIVATE_V4_CLASS_A.networkPrefix, PRIVATE_V4_CLASS_A.subnet, 'IPv4')) {
        return AddressClass.PrivateA
      } else if (inSameNetwork(tuples[0][1], PRIVATE_V4_CLASS_B.networkPrefix, PRIVATE_V4_CLASS_B.subnet, 'IPv4')) {
        return AddressClass.PrivateB
      } else if (inSameNetwork(tuples[0][1], PRIVATE_V4_CLASS_C.networkPrefix, PRIVATE_V4_CLASS_C.subnet, 'IPv4')) {
        return AddressClass.PrivateC
      } else if (isLocalhost(tuples[0][1], 'IPv4')) {
        return AddressClass.Loopback
      } else if (
        inSameNetwork(tuples[0][1], CARRIER_GRADE_NAT_NETWORK.networkPrefix, CARRIER_GRADE_NAT_NETWORK.subnet, 'IPv4')
      ) {
        return AddressClass.CarrierNAT
      } else if (
        !isPrivateAddress(tuples[0][1], 'IPv4') &&
        !isReservedAddress(tuples[0][1], 'IPv4') &&
        !isLinkLocaleAddress(tuples[0][1], 'IPv4')
      ) {
        return AddressClass.Public
      } else {
        return AddressClass.Invalid
      }
    case CODE_IP6:
      if (isLocalhost(tuples[0][1], 'IPv6')) {
        return AddressClass.Loopback6
      } else if (!isLinkLocaleAddress(tuples[0][1], 'IPv6')) {
        return AddressClass.Public6
      } else {
        return AddressClass.Invalid6
      }
    default:
      throw Error(`Invalid addr ${ma.toString()}`)
  }
}

/**
 * Comparator used to sort addresses in local-mode, see `addressPriorityLocal` function
 * @param addrA first Multiaddr
 * @param addrB second Multiaddr
 */
export function compareAddressesLocalMode(addrA: Multiaddr, addrB: Multiaddr): -1 | 0 | 1 {
  const result = addressPriorityLocal(maToClass(addrA)) - addressPriorityLocal(maToClass(addrB))

  if (result == 0) {
    return 0
  } else if (result < 0) {
    return -1
  } else {
    return 1
  }
}

/**
 * Comparator used to sort adresses according to their priority
 * defined by `addressPriorityPublic` function
 * @param addrA first Multiaddr
 * @param addrB second Multiaddr
 */
export function compareAddressesPublicMode(addrA: Multiaddr, addrB: Multiaddr): -1 | 0 | 1 {
  const result = addressPriorityPublic(maToClass(addrA)) - addressPriorityPublic(maToClass(addrB))

  if (result == 0) {
    return 0
  } else if (result < 0) {
    return -1
  } else {
    return 1
  }
}
