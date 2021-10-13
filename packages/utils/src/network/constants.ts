import type { NetworkInterfaceInfo } from 'os'

export type Network = {
  subnet: Uint8Array
  networkPrefix: Uint8Array
  family: NetworkInterfaceInfo['family']
}

export type Address = {
  address: Uint8Array
  family: NetworkInterfaceInfo['family']
}

// Only useful if in same network
export const PRIVATE_NETWORK: Network[] = [
  {
    subnet: Uint8Array.from([255, 0, 0, 0]),
    networkPrefix: Uint8Array.from([0, 0, 0, 0]),
    family: 'IPv4'
  },
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

// Only useful when running > 1 instances on the same host
export const LOOPBACK_ADDRS: Network[] = [
  {
    subnet: Uint8Array.from([255, 0, 0, 0]),
    networkPrefix: Uint8Array.from([127, 0, 0, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 254]),
    networkPrefix: Uint8Array.from([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    family: 'IPv6'
  }
]

export const RESERVED_ADDRS: Network[] = [
  {
    subnet: Uint8Array.from([255, 255, 255, 0]),
    networkPrefix: Uint8Array.from([192, 0, 0, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([255, 255, 255, 0]),
    networkPrefix: Uint8Array.from([192, 0, 2, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([255, 255, 255, 255]),
    networkPrefix: Uint8Array.from([255, 255, 255, 255]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([240, 0, 0, 0]),
    networkPrefix: Uint8Array.from([224, 0, 0, 0]),
    family: 'IPv4'
  },
  {
    subnet: Uint8Array.from([240, 0, 0, 0]),
    networkPrefix: Uint8Array.from([240, 0, 0, 0]),
    family: 'IPv4'
  }
]
