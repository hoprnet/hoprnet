/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/bl.ts" />

import { stringToU8a } from '@hoprnet/hopr-utils'
import type { Stream, StreamType } from 'libp2p'
import type Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import { NetworkInterfaceInfo, networkInterfaces } from 'os'

import { CODE_P2P } from './constants'

type MyStream = AsyncGenerator<StreamType | Buffer | string, void>

export function toU8aStream(source: MyStream): Stream['source'] {
  return (async function* () {
    for await (const msg of source) {
      if (typeof msg === 'string') {
        yield new TextEncoder().encode(msg)
      } else if (Buffer.isBuffer(msg)) {
        yield msg
      } else {
        yield msg.slice()
      }
    }
  })()
}

export function extractPeerIdFromMultiaddr(ma: Multiaddr): PeerId {
  const tuples = ma.stringTuples()

  let destPeerId: string
  if (tuples[0][0] == CODE_P2P) {
    destPeerId = tuples[0][1] as string
  } else if (tuples.length >= 3 && tuples[2][0] == CODE_P2P) {
    destPeerId = tuples[2][1] as string
  } else {
    throw Error(`Invalid Multiaddr. Got ${ma.toString()}`)
  }

  return PeerId.createFromB58String(destPeerId)
}

type AddressFamily = 'IPv4' | 'IPv6' | 'ipv4' | 'ipv6'

export function isAnyAddress(address: string, family: AddressFamily): boolean {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return address === '0.0.0.0'
    case 'ipv6':
      return address === '::'
    default:
      throw Error(`Invalid address family`)
  }
}

export function isLinkLocaleAddress(address: string, family: AddressFamily): boolean {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return (
        address.startsWith('192.168.') ||
        address.startsWith('10.') ||
        address.startsWith('172.16.') ||
        address.startsWith('169.254.') ||
        address.startsWith('100.64')
      )
    case 'ipv6':
      return address.startsWith('fe80:')
    default:
      throw Error(`Invalid address family`)
  }
}

export function isLocalhost(address: string, family: AddressFamily): boolean {
  switch (family.toLowerCase()) {
    case 'ipv4':
      return address === '127.0.0.1'
    case 'ipv6':
      return address === '::1'
    default:
      throw Error(`Invalid address family`)
  }
}

export function ipToU8a(address: string, family: AddressFamily) {
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

export function getNetworkPrefix(address: Uint8Array, subnet: Uint8Array, family: AddressFamily) {
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
  family: AddressFamily
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

export type Network = {
  subnet: Uint8Array
  networkPrefix: Uint8Array
}

function toNetworkPrefix(addr: NetworkInterfaceInfo): Network {
  const subnet = ipToU8a(addr.netmask, addr.family)
  const address = ipToU8a(addr.address, addr.family)

  return {
    subnet,
    networkPrefix: getNetworkPrefix(address, subnet, addr.family)
  }
}

function getAddresses(cond: (address: string, family: 'IPv4' | 'IPv6') => boolean) {
  let result = []

  for (const iface of Object.values(networkInterfaces())) {
    for (const addr of iface ?? []) {
      if (cond(addr.address, addr.family)) {
        result.push(addr)
      }
    }
  }

  return result.map(toNetworkPrefix)
}

export function getLocalAddresses(_iface?: string): Network[] {
  return getAddresses(isLinkLocaleAddress)
}

export function getPublicAddresses(_iface?: string): Network[] {
  return getAddresses(
    (address: string, family: 'IPv4' | 'IPv6') => !isLinkLocaleAddress(address, family) && !isLocalhost(address, family)
  )
}

export function getLocalHosts(_iface?: string): Network[] {
  return getAddresses(isLocalhost)
}
