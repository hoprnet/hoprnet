import type { NetworkInterfaceInfo } from 'os'

export type Network = {
  subnet: Uint8Array
  networkPrefix: Uint8Array
  family: NetworkInterfaceInfo['family']
}

// Only useful if in same network
export const PRIVATE_NETWORK: Network[] = [
  {
    subnet: Uint8Array.from([255, 0, 0, 0]),
    networkPrefix: Uint8Array.from([10, 0, 0, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([255, 240, 0, 0]),
    networkPrefix: Uint8Array.from([172, 16, 0, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([255, 255, 0, 0]),
    networkPrefix: Uint8Array.from([192, 168, 0, 0]),
    family: 'IPv4'
  },
  // Addresses used for carrier-grade NAT, see https://en.wikipedia.org/wiki/IPv4_shared_address_space
  {
    networkPrefix: Uint8Array.from([255, 240, 0, 0]),
    subnet: Uint8Array.from([100, 64, 0, 0]),
    family: 'IPv4'
  }
]

// Link-local addresses are not routable
export const LINK_LOCAL_NETWORKS: Network[] = [
  {
    subnet: Uint8Array.from([255, 255, 255, 0]),
    networkPrefix: Uint8Array.from([169, 254, 0, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([254, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    networkPrefix: Uint8Array.from([254, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    family: 'IPv6'
  }
]
