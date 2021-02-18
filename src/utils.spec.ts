import {
  ipToU8a,
  getNetworkPrefix,
  inSameNetwork /* getLocalAddresses, getLocalHosts, getPublicAddresses */
} from './utils'
// import { u8aEquals } from '@hoprnet/hopr-utils'
import assert from 'assert'
import { u8aEquals } from '@hoprnet/hopr-utils'

describe('test utils', function () {
  it.skip('should convert ip addresses', function () {
    console.log(ipToU8a('fe80::1', 'IPv6'))

    console.log(ipToU8a('::1', 'IPv6'))

    console.log(ipToU8a('2001:0db8:0000:0000:0000:8a2e:0370:7334', 'IPv6'))

    console.log(ipToU8a('ffff:ffff:ffff:ffff::', 'ipv6'))

    console.log(ipToU8a('2001:0db8:0000:0000:0000:8a2e:0370:7334', 'ipv6'))

    // @TODO
    // console.log(ipToU8a('2001:db8::8a2e:370:7334', 'IPv6'))
  })

  it('should return a network prefix', function () {
    const address4 = ipToU8a('192.168.1.23', 'ipv4')

    const subnet4_1 = ipToU8a('255.255.255.0', 'ipv4')
    const subnet4_2 = ipToU8a('255.255.254.0', 'ipv4')

    assert(u8aEquals(Uint8Array.from([192, 168, 1, 0]), getNetworkPrefix(address4, subnet4_1, 'ipv4')))

    assert(u8aEquals(Uint8Array.from([192, 168, 0, 0]), getNetworkPrefix(address4, subnet4_2, 'ipv4')))

    const address6 = ipToU8a('2001:0db8:0000:0000:0000:8a2e:0371:7334', 'ipv6')

    const subnet6_1 = ipToU8a('ffff:ffff:ffff:ffff:ffff:ffff:ffff::', 'ipv6')
    const subnet6_2 = ipToU8a('ffff:ffff:ffff:ffff:ffff:ffff:fffe::', 'ipv6')

    assert(
      u8aEquals(
        Uint8Array.from([32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 138, 46, 3, 113, 0, 0]),
        getNetworkPrefix(address6, subnet6_1, 'ipv6')
      )
    )

    assert(
      u8aEquals(
        Uint8Array.from([32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 138, 46, 3, 112, 0, 0]),
        getNetworkPrefix(address6, subnet6_2, 'ipv6')
      )
    )
  })

  it('should be in subnet', function () {
    const address = ipToU8a('192.0.2.130', 'ipv4')
    const subnet = ipToU8a('255.255.255.0', 'ipv4')

    const networkPrefix = getNetworkPrefix(address, subnet, 'ipv4')

    assert(!inSameNetwork(ipToU8a('192.0.1.131', 'ipv4'), networkPrefix, subnet, 'ipv4'))

    assert(inSameNetwork(ipToU8a('192.0.2.131', 'ipv4'), networkPrefix, subnet, 'ipv4'))

    assert(!inSameNetwork(ipToU8a('192.0.3.131', 'ipv4'), networkPrefix, subnet, 'ipv4'))
  })

  //   it('should get my addresses in a structured way', function () {
  //     console.log(`localHost`, getLocalHosts())
  //     console.log(`localAddresses`, getLocalAddresses())
  //     console.log(`public addresses`, getPublicAddresses())
  //   })
})
