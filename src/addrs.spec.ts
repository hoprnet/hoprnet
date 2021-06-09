import { getAddrs } from './addrs'
import PeerId from 'peer-id'
import assert from 'assert'
import { networkInterfaces } from 'os'

describe('addrs', function () {
  let pId: PeerId
  const INVALID_NETWORK_INTERFACE = Object.keys(networkInterfaces()).join().concat(`foo`)
  const VALID_NETWORK_INTERFACE = Object.keys(networkInterfaces())[0]

  before(async function () {
    pId = await PeerId.create({ keyType: 'secp256k1' })
  })

  it('should understand network interface', function () {
    assert(
      getAddrs(9091, pId.toB58String(), {
        interface: INVALID_NETWORK_INTERFACE,
        useIPv4: true,
        includeLocalhostIPv4: true
      }).length == 0,
      'Should not output any addresses if the specified network interface does not exist'
    )

    assert(
      getAddrs(9091, pId.toB58String(), {
        interface: VALID_NETWORK_INTERFACE,
        useIPv4: true,
        useIPv6: true,
        includeLocalhostIPv4: true,
        includeLocalhostIPv6: true,
        includePrivateIPv4: true
      }).length >= 1,
      'Should output at least one address if the specified network interface exists'
    )
  })

  it('should get ip address', function () {
    assert(
      getAddrs(
        9091,
        pId.toB58String(),
        {
          useIPv6: true,
          useIPv4: true,
          includeLocalhostIPv4: true,
          includeLocalhostIPv6: true,
          includePrivateIPv4: true
        },
        {
          myFakeInterface: [
            {
              address: 'fe80::cdc2:2079:792:3d33',
              netmask: 'ffff:ffff:ffff:ffff::',
              family: 'IPv6'
            } as any
          ]
        }
      ).length == 0,
      `Should not use link-locale addresses`
    )

    assert(
      getAddrs(
        9091,
        pId.toB58String(),
        {
          useIPv4: true
        },
        {
          myFakeInterface: [
            {
              address: '10.0.27.191',
              netmask: '255.0.0.0',
              family: 'IPv4'
            } as any
          ]
        }
      ).length == 0,
      `Should not include private IPv4 addresses`
    )

    assert(
      getAddrs(
        9091,
        pId.toB58String(),
        {
          useIPv4: true
        },
        {
          myFakeInterface: [
            {
              address: '2001:db8:0:0:0:0:1428:57ab',
              netmask: 'ffff:ffff:ffff:ffff::',
              family: 'IPv6'
            } as any
          ]
        }
      ).length == 0,
      `Should not include IPv6 addresses when searching for IPv4 addresses`
    )
  })

  it('try configuration edge cases', function () {
    assert.throws(() =>
      getAddrs(12345, pId.toB58String(), {
        useIPv4: false,
        includeLocalhostIPv4: true
      })
    )

    assert.throws(() =>
      getAddrs(12345, pId.toB58String(), {
        useIPv6: false,
        includeLocalhostIPv6: true
      })
    )

    assert.throws(() => getAddrs(12345, pId.toB58String(), {}))
  })
})
