import { stringToU8a, u8aToHex } from '..'
import type { Network } from './constants'
import { PRIVATE_NETWORK, LINK_LOCAL_NETWORKS, LOOPBACK_ADDRS, RESERVED_ADDRS } from './constants'

import type { NetworkInterfaceInfo } from 'os'
import { networkInterfaces } from 'os'

/**
 * Checks if given address is any address
 * @param address ip address to check
 * @param family ip address family, 'IPv4' or 'IPv6'
 */
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

/**
 * Checks if given address is a loopback address (localhost)
 * @param address ip address to check
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns true if localhost
 */
export function isLocalhost(address: Uint8Array, family: NetworkInterfaceInfo['family']): boolean {
  return checkNetworks(LOOPBACK_ADDRS, address, family)
}

/**
 * Checks if given address is a private address
 * @param address ip address to check
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns true if private address
 */
export function isPrivateAddress(address: Uint8Array, family: NetworkInterfaceInfo['family']): boolean {
  return checkNetworks(PRIVATE_NETWORK, address, family)
}

/**
 * Checks if given address is link-locale address
 * @param address ip address to check
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns true if is link-locale address
 */
export function isLinkLocaleAddress(address: Uint8Array, family: NetworkInterfaceInfo['family']): boolean {
  return checkNetworks(LINK_LOCAL_NETWORKS, address, family)
}

/**
 * Checks if given address is a reserved address
 * @param address ip address to check
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns true if address is a reserved address
 */
export function isReservedAddress(address: Uint8Array, family: NetworkInterfaceInfo['family']): boolean {
  return checkNetworks(RESERVED_ADDRS, address, family)
}

/**
 * Checks if given address is in one of the given networks
 * @dev Used to check if a node is in the same network
 * @param networks network address spaces to check
 * @param address ip address to check
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns true if address is at least one of the given networks
 */
export function checkNetworks(
  networks: Network[],
  address: Uint8Array,
  family: NetworkInterfaceInfo['family']
): boolean {
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

/**
 * Converts ip address string to Uint8Arrays
 * @param address ip address as string, e.g. 192.168.12.34
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns Byte representation of the given ip address
 */
export function ipToU8aAddress(address: string, family: NetworkInterfaceInfo['family']): Uint8Array {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return Uint8Array.from(address.split('.').map((x) => parseInt(x)))
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

/**
 * Converts ip address from byte representation to string
 * @param address ip addr given as Uint8Array
 * @param family ip address family, 'IPv4' or 'IPv6'
 * @returns
 */
export function u8aAddrToString(address: Uint8Array, family: NetworkInterfaceInfo['family']) {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return address.join('.')
    case 'ipv6':
      let result = ''
      for (let i = 0; i < 8; i++) {
        result += u8aToHex(address.subarray(i * 2, i * 2 + 2), false)

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

function getAddresses(cond: (address: Uint8Array, family: NetworkInterfaceInfo['family']) => boolean): Network[] {
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

export function getPrivateAddresses(_iface?: string) {
  return getAddresses(isPrivateAddress)
}
export function getLocalAddresses(_iface?: string): Network[] {
  return getAddresses(isLinkLocaleAddress)
}

export function getPublicAddresses(_iface?: string): Network[] {
  return getAddresses(
    (address: Uint8Array, family: NetworkInterfaceInfo['family']) =>
      !isPrivateAddress(address, family) && !isLinkLocaleAddress(address, family) && !isLocalhost(address, family)
  )
}

export function getLocalHosts(_iface?: string): Network[] {
  return getAddresses(isLocalhost)
}
