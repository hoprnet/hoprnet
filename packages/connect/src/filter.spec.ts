import type { Network } from '@hoprnet/hopr-utils'

import { Multiaddr } from 'multiaddr'
import { Filter } from './filter'
import assert from 'assert'
import { toNetworkPrefix, privKeyToPeerId } from '@hoprnet/hopr-utils'

let firstPeer = privKeyToPeerId(`0x22f7c3c101db7a73c42d3adecbd2700173f19a249b5ef115c25020b091822083`)
let secondPeer = privKeyToPeerId(`0xbb25701334f6f989ab51322d0064b3755fc3a65770e4a240df163c355bd8cd26`)
let thirdPeer = privKeyToPeerId(`0x175590e95d378e66572e09bc9d8badffe087ae962fc7551f17380293d1ca2fc5`)

class TestFilter extends Filter {
  /**
   * THIS METHOD IS ONLY USED FOR TESTING
   * @dev Used to set falsy local network
   * @param mAddrs new local addresses
   */
  _setLocalAddressesForTesting(networks: Network[]): void {
    this.myPrivateNetworks = networks
  }
}

describe('test addr filtering', function () {
  let filter: TestFilter

  beforeEach(function () {
    filter = new TestFilter(firstPeer)
  })

  it('should accept valid circuit addresses', function () {
    assert(
      filter.filter(new Multiaddr(`/p2p/${firstPeer.toB58String()}`)) == false,
      'Should not accept relay addrs without recipient'
    )

    assert(
      filter.filter(new Multiaddr(`/p2p/${firstPeer.toB58String()}/p2p-circuit/p2p/${firstPeer.toB58String()}`)) ==
        false,
      'Should not accept relay circuits that include own address'
    )

    assert(
      filter.filter(new Multiaddr(`/p2p/${secondPeer.toB58String()}/p2p-circuit/p2p/${secondPeer.toB58String()}`)) ==
        false,
      'Should not accept loopbacks'
    )

    filter.setAddrs(
      [new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(
      filter.filter(new Multiaddr(`/p2p/${secondPeer.toB58String()}/p2p-circuit/p2p/${thirdPeer.toB58String()}`)) ==
        true
    )
  })

  it('refuse listening to bad addresses', function () {
    assert(filter.filter(new Multiaddr(`/ip4/1.1.1.1/udp/123`)) == false, 'Should not accept udp addresses')

    assert(
      filter.filter(new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${secondPeer.toB58String()}`)) == false,
      'Should not listen to other peerIds'
    )
  })

  it('set addresses', function () {
    assert(!filter.addrsSet)

    filter.setAddrs(
      [new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)
  })

  it('refuse dialing IPv4 when listening to IPv6', function () {
    filter.setAddrs(
      [new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    assert(
      filter.filter(new Multiaddr(`/ip6/::1/tcp/1/p2p/${secondPeer.toB58String()}`)) == false,
      'Refuse dialing IPv6'
    )
  })

  it('refuse dialing IPv6 when listening to IPv4', function () {
    filter.setAddrs(
      [new Multiaddr(`/ip6/::1/tcp/123/p2p/${firstPeer.toB58String()}`)],
      [new Multiaddr(`/ip6/::/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    assert(
      filter.filter(new Multiaddr(`/ip4/1.1.1.1/tcp/1/p2p/${secondPeer.toB58String()}`)) == false,
      'Refuse dialing IPv4'
    )
  })

  it('understand dual-stack', function () {
    filter.setAddrs(
      [
        new Multiaddr(`/ip6/::1/tcp/123/p2p/${firstPeer.toB58String()}`),
        new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)
      ],
      [
        new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`),
        new Multiaddr(`/ip6/::/tcp/0/p2p/${firstPeer.toB58String()}`)
      ]
    )

    assert(filter.addrsSet)

    assert(
      filter.filter(new Multiaddr(`/ip4/1.1.1.1/tcp/1/p2p/${secondPeer.toB58String()}`)) == true,
      'Refuse dialing IPv4'
    )

    assert(
      filter.filter(new Multiaddr(`/ip6/::1/tcp/1/p2p/${secondPeer.toB58String()}`)) == true,
      'Refuse dialing IPv6'
    )
  })

  it(`dial on same host`, function () {
    filter.setAddrs(
      [
        // localhost
        new Multiaddr(`/ip4/127.0.0.1/tcp/2/p2p/${firstPeer.toB58String()}`),
        // private address
        new Multiaddr(`/ip4/10.0.0.1/tcp/2/p2p/${firstPeer.toB58String()}`),
        // link-locale address
        new Multiaddr(`/ip4/169.254.0.1/tcp/2/p2p/${firstPeer.toB58String()}`),
        // public address
        new Multiaddr(`/ip4/1.2.3.4/tcp/2/p2p/${firstPeer.toB58String()}`)
      ],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    // localhost
    assert(filter.filter(new Multiaddr(`/ip4/127.0.0.1/tcp/1/p2p/${secondPeer.toB58String()}`)) == true)
    assert(filter.filter(new Multiaddr(`/ip4/127.0.0.1/tcp/2/p2p/${secondPeer.toB58String()}`)) == false)

    // private address
    assert(filter.filter(new Multiaddr(`/ip4/10.0.0.1/tcp/1/p2p/${secondPeer.toB58String()}`)) == true)
    assert(filter.filter(new Multiaddr(`/ip4/10.0.0.1/tcp/2/p2p/${secondPeer.toB58String()}`)) == false)

    // link-locale address
    assert(filter.filter(new Multiaddr(`/ip4/169.254.0.1/tcp/1/p2p/${secondPeer.toB58String()}`)) == true)
    assert(filter.filter(new Multiaddr(`/ip4/169.254.0.1/tcp/2/p2p/${secondPeer.toB58String()}`)) == false)

    // public address
    assert(filter.filter(new Multiaddr(`/ip4/1.2.3.4/tcp/1/p2p/${secondPeer.toB58String()}`)) == true)
    assert(filter.filter(new Multiaddr(`/ip4/1.2.3.4/tcp/2/p2p/${secondPeer.toB58String()}`)) == false)
  })

  it('self-dial', function () {
    filter.setAddrs(
      [new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    assert(filter.filter(new Multiaddr(`/ip4/127.0.0.1/tcp/1/p2p/${firstPeer.toB58String()}`)) == false)

    assert(
      filter.filter(new Multiaddr(`/p2p/${secondPeer.toB58String()}/p2p-circuit/p2p/${firstPeer.toB58String()}`)) ==
        false
    )
    assert(
      filter.filter(new Multiaddr(`/p2p/${firstPeer.toB58String()}/p2p-circuit/p2p/${secondPeer.toB58String()}`)) ==
        false
    )
  })

  it('invalid addresses & invalid ports', function () {
    filter.setAddrs(
      [new Multiaddr(`/ip4/1.1.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    assert(filter.filter(new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${secondPeer.toB58String()}`)) == false)
  })

  it('link-locale addresses', function () {
    filter.setAddrs(
      [
        new Multiaddr(`/ip4/10.0.0.1/tcp/123/p2p/${firstPeer.toB58String()}`),
        new Multiaddr(`/ip4/192.168.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)
      ],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    assert(filter.filter(new Multiaddr(`/ip4/169.254.0.1/tcp/2/p2p/${secondPeer.toB58String()}`)) == false)
  })

  it('local networks', function () {
    filter.setAddrs(
      [
        new Multiaddr(`/ip4/10.0.0.1/tcp/123/p2p/${firstPeer.toB58String()}`),
        new Multiaddr(`/ip4/192.168.1.1/tcp/123/p2p/${firstPeer.toB58String()}`)
      ],
      [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${firstPeer.toB58String()}`)]
    )

    assert(filter.addrsSet)

    filter._setLocalAddressesForTesting([
      toNetworkPrefix({
        address: '10.0.0.1',
        netmask: '255.0.0.0',
        family: 'IPv4'
      } as any),
      toNetworkPrefix({
        address: '192.168.1.0',
        netmask: '255.255.255.0',
        family: 'IPv4'
      } as any)
    ])

    assert(filter.filter(new Multiaddr(`/ip4/10.0.0.2/tcp/1/p2p/${secondPeer.toB58String()}`)) == true)
    assert(filter.filter(new Multiaddr(`/ip4/192.168.1.2/tcp/1/p2p/${secondPeer.toB58String()}`)) == true)

    assert(filter.filter(new Multiaddr(`/ip4/192.168.0.1/tcp/1/p2p/${secondPeer.toB58String()}`)) == false)
  })
})
