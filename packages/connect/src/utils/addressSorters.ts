import type { Multiaddr } from 'multiaddr'

import { CODE_IP4, CODE_IP6, CODE_P2P } from '../constants'
import {
  inSameNetwork,
  isLinkLocaleAddress,
  isLocalhost,
  isPrivateAddress,
  isReservedAddress,
  PRIVATE_V4_CLASS_A,
  PRIVATE_V4_CLASS_B,
  PRIVATE_V4_CLASS_C,
  CARRIER_GRADE_NAT_NETWORK
} from '@hoprnet/hopr-utils'

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
    case AddressClass.PrivateA:
      return 0
    case AddressClass.PrivateB:
      return 1
    case AddressClass.PrivateC:
      return 2
    case AddressClass.Loopback:
      return 3
    case AddressClass.Loopback6:
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

export function compareAddressesLocalMode(addrA: Multiaddr, addrB: Multiaddr): number {
  return addressPriorityLocal(maToClass(addrA)) - addressPriorityLocal(maToClass(addrB))
}

export function compareAddressesPublicMode(addrA: Multiaddr, addrB: Multiaddr): number {
  return addressPriorityPublic(maToClass(addrA)) - addressPriorityPublic(maToClass(addrB))
}
