import { stringToU8a, u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'
import type { Network } from './constants'
import { PRIVATE_NETWORK, LINK_LOCAL_NETWORKS, LOCALHOST_ADDRS } from './constants'

import { NetworkInterfaceInfo, networkInterfaces } from 'os'

export function isAnyAddress(address: string, family: NetworkInterfaceInfo['family']): boolean {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return address === '0.0.0.0'
    case 'ipv6':
      return address === '::'
    default:
      throw Error(`Invalid address family`)
  }
}

export function isLocalhost(address: Uint8Array, family: NetworkInterfaceInfo['family']) {
  for (const addr of LOCALHOST_ADDRS) {
    if (addr.family === family && u8aEquals(address, addr.address)) {
      return true
    }
  }
  return false
}

export function isPrivateAddress(address: Uint8Array, family: NetworkInterfaceInfo['family']) {
  return checkNetworks(PRIVATE_NETWORK, address, family)
}

export function isLinkLocaleAddress(address: Uint8Array, family: NetworkInterfaceInfo['family']) {
  return checkNetworks(LINK_LOCAL_NETWORKS, address, family)
}

export function checkNetworks(networks: Network[], address: Uint8Array, family: NetworkInterfaceInfo['family']) {
  for (const networkAddress of networks) {
    if (
      networkAddress.family === family &&
      inSameNetwork(address, networkAddress.networkPrefix, networkAddress.subnet, family)
    ) {
      return true
    }
  }
  return false
}

export function ipToU8aAddress(address: string, family: NetworkInterfaceInfo['family']) {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return Uint8Array.from(address.split('.').map((x: string) => parseInt(x)))
    case 'ipv6':
      const splitted = address.split(':')
      if (address.endsWith(':')) {
        splitted[splitted.length - 1] = '0'
      }

      if (address.startsWith(':')) {
        splitted[0] = '0'
      }

      if (splitted.some((x) => x.length == 0)) {
        splitted.splice(
          splitted.findIndex((x) => x.length == 0),
          1,
          ...Array.from({ length: 8 - splitted.length + 1 }, () => '0')
        )
      }

      const result = new Uint8Array(16)

      for (const [index, str] of splitted.entries()) {
        result.set(stringToU8a(str.padStart(4, '0'), 2), index * 2)
      }

      return result
    default:
      throw Error(`Invalid address family`)
  }
}

export function u8aAddrToString(address: Uint8Array, family: NetworkInterfaceInfo['family']) {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return address.join('.')
    case 'ipv6':
      let result = ''
      for (let i = 0; i < 8; i++) {
        result += u8aToHex(address.subarray(i * 2, i * 2 +2), false)

        if (i != 7) {
          result += ':'
        }
      }
      return result
    default:
      throw Error('Invalid address family.')
  }

}

export function getNetworkPrefix(address: Uint8Array, subnet: Uint8Array, family: NetworkInterfaceInfo['family']) {
  const filler = (_: any, index: number) => subnet[index] & address[index]

  switch (family.toLowerCase()) {
    case 'ipv4':
      return Uint8Array.from(new Uint8Array(4), filler)
    case 'ipv6':
      return Uint8Array.from(new Uint8Array(16), filler)
    default:
      throw Error(`Invalid address family`)
  }
}

export function inSameNetwork(
  address: Uint8Array,
  networkPrefix: Uint8Array,
  subnetMask: Uint8Array,
  family: NetworkInterfaceInfo['family']
): boolean {
  const checkLength = (length: number) =>
    address.length == length && networkPrefix.length == length && subnetMask.length == length

  switch (family.toLowerCase()) {
    case 'ipv4':
      if (!checkLength(4)) {
        throw Error('Invalid length')
      }
      break
    case 'ipv6':
      if (!checkLength(16)) {
        throw Error('Invalid length')
      }
      break
    default:
      throw Error(`Invalid address family`)
  }

  for (const [index, el] of address.entries()) {
    if ((el & subnetMask[index]) != networkPrefix[index]) {
      return false
    }
  }

  return true
}

function toNetworkPrefix(addr: NetworkInterfaceInfo): Network {
  const subnet = ipToU8aAddress(addr.netmask, addr.family)
  const address = ipToU8aAddress(addr.address, addr.family)

  return {
    subnet,
    networkPrefix: getNetworkPrefix(address, subnet, addr.family),
    family: addr.family
  }
}

function getAddresses(cond: (address: Uint8Array, family: 'IPv4' | 'IPv6') => boolean): Network[] {
  let result = []

  for (const iface of Object.values(networkInterfaces())) {
    for (const addr of iface ?? []) {
      const networkPrefix = toNetworkPrefix(addr)
      if (cond(ipToU8aAddress(addr.address, addr.family), addr.family)) {
        result.push(networkPrefix)
      }
    }
  }

  return result
}

export function getLocalAddresses(_iface?: string): Network[] {
  return getAddresses(isLinkLocaleAddress)
}

export function getPublicAddresses(_iface?: string): Network[] {
  return getAddresses(
    (address: Uint8Array, family: 'IPv4' | 'IPv6') => !isLinkLocaleAddress(address, family) && !isLocalhost(address, family)
  )
}

export function getLocalHosts(_iface?: string): Network[] {
  return getAddresses(isLocalhost)
}
