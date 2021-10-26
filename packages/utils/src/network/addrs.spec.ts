import {
  ipToU8aAddress,
  getNetworkPrefix,
  inSameNetwork,
  u8aAddrToString,
  getPrivateAddresses,
  isPrivateAddress,
  getLocalAddresses,
  getLocalHosts,
  isLinkLocaleAddress,
  isLocalhost,
  getPublicAddresses
} from './addrs'
import type { Network } from './constants'
import { u8aEquals } from '..'
import assert from 'assert'

describe('test utils', function () {
  it('should convert ip addresses', function () {
    assert(u8aEquals(Uint8Array.from([1, 1, 1, 1]), ipToU8aAddress('1.1.1.1', 'IPv4')))

    assert(u8aEquals(Uint8Array.from([1, 1, 0, 1]), ipToU8aAddress('1.1.0.1', 'IPv4')))

    assert(u8aEquals(Uint8Array.from([0, 0, 0, 0]), ipToU8aAddress('0.0.0.0', 'IPv4')))

    assert(
      u8aEquals(
        Uint8Array.from([254, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        ipToU8aAddress('fe80::1', 'IPv6')
      )
    )

    assert(u8aEquals(Uint8Array.from([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]), ipToU8aAddress('::1', 'IPv6')))

    assert(
      u8aEquals(
        Uint8Array.from([255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0]),
        ipToU8aAddress('ffff:ffff:ffff:ffff::', 'IPv6')
      )
    )

    const testAddress = Uint8Array.from([32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 138, 46, 3, 112, 115, 52])

    assert(u8aEquals(testAddress, ipToU8aAddress('2001:0db8:0000:0000:0000:8a2e:0370:7334', 'IPv6')))

    assert(u8aEquals(testAddress, ipToU8aAddress('2001:0db8:00:000:0000:8a2e:370:7334', 'IPv6')))

    assert(u8aEquals(testAddress, ipToU8aAddress('2001:db8::8a2e:370:7334', 'IPv6')))

    assert(
      u8aEquals(
        Uint8Array.from([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0]),
        ipToU8aAddress('ffff:ffff:ffff:ffff:ffff:ffff:ffff::', 'IPv6')
      )
    )

    assert(
      u8aEquals(
        Uint8Array.from([0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]),
        ipToU8aAddress('::ffff:ffff:ffff:ffff:ffff:ffff:ffff', 'IPv6')
      )
    )
  })

  it('should return a network prefix', function () {
    const address4 = ipToU8aAddress('192.168.1.23', 'IPv4')

    const subnet4_1 = ipToU8aAddress('255.255.255.0', 'IPv4')
    const subnet4_2 = ipToU8aAddress('255.255.254.0', 'IPv4')

    assert(u8aEquals(Uint8Array.from([192, 168, 1, 0]), getNetworkPrefix(address4, subnet4_1, 'IPv4')))

    assert(u8aEquals(Uint8Array.from([192, 168, 0, 0]), getNetworkPrefix(address4, subnet4_2, 'IPv4')))

    const address6 = ipToU8aAddress('2001:0db8:0000:0000:0000:8a2e:0371:7334', 'IPv6')

    const subnet6_1 = ipToU8aAddress('ffff:ffff:ffff:ffff:ffff:ffff:ffff::', 'IPv6')
    const subnet6_2 = ipToU8aAddress('ffff:ffff:ffff:ffff:ffff:ffff:fffe::', 'IPv6')

    assert(
      u8aEquals(
        Uint8Array.from([32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 138, 46, 3, 113, 0, 0]),
        getNetworkPrefix(address6, subnet6_1, 'IPv6')
      )
    )

    assert(
      u8aEquals(
        Uint8Array.from([32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 138, 46, 3, 112, 0, 0]),
        getNetworkPrefix(address6, subnet6_2, 'IPv6')
      )
    )
  })

  it('should be in subnet', function () {
    const address = ipToU8aAddress('192.0.2.130', 'IPv4')
    const subnet = ipToU8aAddress('255.255.255.0', 'IPv4')

    const networkPrefix = getNetworkPrefix(address, subnet, 'IPv4')

    assert(!inSameNetwork(ipToU8aAddress('192.0.1.131', 'IPv4'), networkPrefix, subnet, 'IPv4'))

    assert(inSameNetwork(ipToU8aAddress('192.0.2.131', 'IPv4'), networkPrefix, subnet, 'IPv4'))

    assert(!inSameNetwork(ipToU8aAddress('192.0.3.131', 'IPv4'), networkPrefix, subnet, 'IPv4'))
  })

  it('should convert u8aAddr back to string', function () {
    assert(
      '0000:ffff:ffff:ffff:ffff:ffff:ffff:ffff' ===
        u8aAddrToString(
          Uint8Array.from([0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255]),
          'IPv6'
        )
    )

    assert(
      'ffff:ffff:ffff:ffff:ffff:ffff:ffff:0000' ===
        u8aAddrToString(
          Uint8Array.from([255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0]),
          'IPv6'
        )
    )
  })

  it('should detect private networks', function () {
    assert(isPrivateAddress(ipToU8aAddress('192.168.1.131', 'IPv4'), 'IPv4'))
    assert(isPrivateAddress(ipToU8aAddress('10.0.27.191', 'IPv4'), 'IPv4'))
    assert(!isPrivateAddress(ipToU8aAddress('172.15.0.131', 'IPv4'), 'IPv4'))
  })

  it('should detect local addresses', function () {
    assert(isLocalhost(ipToU8aAddress('127.0.0.1', 'IPv4'), 'IPv4'))
    assert(isLocalhost(ipToU8aAddress('::1', 'IPv6'), 'IPv6'))
  })

  it('should detect local addresses as local', function () {
    assert(getLocalHosts().every((network: Network) => isLocalhost(network.networkPrefix, network.family)))
    assert(getPrivateAddresses().every((network: Network) => isPrivateAddress(network.networkPrefix, network.family)))
    assert(getLocalAddresses().every((network: Network) => isLinkLocaleAddress(network.networkPrefix, network.family)))
    assert(
      getPublicAddresses().every(
        (network: Network) =>
          !isLocalhost(network.networkPrefix, network.family) &&
          !isPrivateAddress(network.networkPrefix, network.family) &&
          !isLinkLocaleAddress(network.networkPrefix, network.family)
      )
    )
  })
})
