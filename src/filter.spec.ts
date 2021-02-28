import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import { Filter } from './filter'
import assert from 'assert'

describe('test addr filtering', function () {
  let firstPeer: PeerId, secondPeer: PeerId
  let filter: Filter

  before(async function () {
    firstPeer = await PeerId.create({ keyType: 'secp256k1' })
    secondPeer = await PeerId.create({ keyType: 'secp256k1' })
  })

  beforeEach(function () {
    filter = new Filter(firstPeer)
  })

  it('should accept valid circuit addresses', function () {
    assert(
      filter.filter(Multiaddr(`/p2p/${firstPeer.toB58String()}`)) == false,
      'Should not accept relay addrs without recipient'
    )

    assert(
      filter.filter(Multiaddr(`/p2p/${firstPeer.toB58String()}/p2p-circuit/p2p/${firstPeer.toB58String()}`)) == false,
      'Should not accept relay circuits that include own address'
    )

    assert(
      filter.filter(Multiaddr(`/p2p/${secondPeer.toB58String()}/p2p-circuit/p2p/${secondPeer.toB58String()}`)) == false,
      'Should not accept loopbacks'
    )

    assert(
      filter.filter(Multiaddr(`/p2p/${secondPeer.toB58String()}/p2p-circuit/p2p/${firstPeer.toB58String()}`)) == true,
      'Should accept proper circuits'
    )
  })

  it('should accept valid ip addresses', function () {
    assert(filter.filter(Multiaddr(`/ip4/1.1.1.1/udp/123`)) == false, 'Should not accept udp addresses')

    assert(filter.filter(Multiaddr(`/ip6/::1/udp/123`)) == false, 'Should not accept udp addresses')

    assert(filter.filter(Multiaddr(`/ip4/1.1.1.1/tcp/123`)) == true, 'Should not accept udp addresses')

    assert(filter.filter(Multiaddr(`/ip6/::1/tcp/123`)) == true, 'Should accept tcp addresses')

    assert(
      filter.filter(Multiaddr(`/ip4/1.1.1.1/tcp/0`)) == true,
      'Should not accept invalid ports before initialization'
    )

    assert(filter.filter(Multiaddr(`/ip6/::1/tcp/0`)) == true, 'Should not accept invalid ports before initialization')

    filter.setAddrs([], [])

    assert(filter.filter(Multiaddr(`/ip4/1.1.1.1/tcp/0`)) == false, 'Should not accept invalid ports')

    assert(filter.filter(Multiaddr(`/ip6/::1/tcp/0`)) == false, 'Should not accept invalid ports')
  })

  it('should understand to which address families the node is listening', function () {
    filter.setAddrs([], [Multiaddr(`/ip4/1.1.1.1/tcp/1`)])

    assert(filter.filter(Multiaddr(`/ip4/1.1.1.1/tcp/1`)) == true, 'Should accept IPv4 when listening to IPv4')

    assert(filter.filter(Multiaddr(`/ip6/::1/tcp/1`)) == false, 'Should not accept IPv6 when listening to IPv4')

    filter.setAddrs([], [Multiaddr(`/ip6/::1/tcp/1`)])

    assert(filter.filter(Multiaddr(`/ip4/1.1.1.1/tcp/1`)) == false, 'Should not accept IPv4 when listening to IPv6')

    assert(filter.filter(Multiaddr(`/ip6/::1/tcp/1`)) == true, 'Should accept IPv6 when listening to IPv6')

    filter.setAddrs([], [Multiaddr(`/ip4/1.1.1.1/tcp/1`), Multiaddr(`/ip6/::1/tcp/1`)])

    assert(filter.filter(Multiaddr(`/ip4/1.1.1.1/tcp/1`)) == true, 'Should not accept IPv4 when listening to IPv6')

    assert(filter.filter(Multiaddr(`/ip6/::1/tcp/1`)) == true, 'Should accept IPv6 when listening to IPv6')
  })

  it('should detect attempts dial ourself', function () {
    filter._setLocalAddressesForTesting([
      {
        subnet: Uint8Array.from([255, 240, 0, 0]),
        networkPrefix: Uint8Array.from([172, 16, 0, 0]),
        family: 'IPv4'
      }
    ])

    filter.setAddrs(
      [
        Multiaddr(`/ip4/127.0.0.1/tcp/1/p2p/16Uiu2HAm26xs51THkoJkjbBG4HVRWt7wQYNkmouNctotkPCbANYv`),
        Multiaddr(`/ip4/172.17.0.1/tcp/1/p2p/16Uiu2HAm26xs51THkoJkjbBG4HVRWt7wQYNkmouNctotkPCbANYv`),
        Multiaddr(`/ip6/2001:db8::8a2e:370:7334/tcp/1/p2p/16Uiu2HAm26xs51THkoJkjbBG4HVRWt7wQYNkmouNctotkPCbANYv`),
        Multiaddr(`/ip4/203.0.113.16/tcp/1/p2p/16Uiu2HAm26xs51THkoJkjbBG4HVRWt7wQYNkmouNctotkPCbANYv`)
      ],
      [Multiaddr(`/ip4/1.1.1.1/tcp/1`), Multiaddr(`/ip6/::1/tcp/1`)]
    )

    assert(filter.filter(Multiaddr(`/ip4/127.0.0.1/tcp/1`)) == false, `Should not dial own address`)

    assert(filter.filter(Multiaddr(`/ip4/127.0.0.1/tcp/2`)) == true, `Should dial on different port on localhost`)

    assert(filter.filter(Multiaddr(`/ip4/172.17.0.1/tcp/1`)) == false, `Should not dial own address`)

    assert(filter.filter(Multiaddr(`/ip4/172.17.0.1/tcp/2`)) == true, `Should dial on different on same local address`)

    assert(filter.filter(Multiaddr(`/ip4/203.0.113.16/tcp/1`)) == false, `Should not dial own address`)

    assert(
      filter.filter(Multiaddr(`/ip4/203.0.113.16/tcp/2`)) == true,
      `Should dial on different on same public address`
    )

    assert(filter.filter(Multiaddr(`/ip6/2001:db8::8a2e:370:7334/tcp/1`)) == false, `Should not dial own address`)

    assert(
      filter.filter(Multiaddr(`/ip6/2001:db8::8a2e:370:7334/tcp/2`)) == true,
      `Should dial on different on same public address`
    )
  })

  it('should understand private networks', function () {
    filter._setLocalAddressesForTesting([
      {
        subnet: Uint8Array.from([255, 240, 0, 0]),
        networkPrefix: Uint8Array.from([172, 16, 0, 0]),
        family: 'IPv4'
      }
    ])

    filter.setAddrs(
      [Multiaddr(`/ip4/172.17.0.1/tcp/1/p2p/16Uiu2HAm26xs51THkoJkjbBG4HVRWt7wQYNkmouNctotkPCbANYv`)],
      [Multiaddr(`/ip4/1.1.1.1/tcp/1`)]
    )

    assert(filter.filter(Multiaddr(`/ip4/172.17.0.2/tcp/1`)) == true, `Should dial addresses in same local network`)

    assert(
      filter.filter(Multiaddr(`/ip4/172.18.0.2/tcp/1`)) == true,
      `Should dial addresses in same local network while respecting subnet`
    )

    assert(
      filter.filter(Multiaddr(`/ip4/192.168.0.2/tcp/1`)) == false,
      `Should not dial local addresses in different local network`
    )

    assert(
      filter.filter(Multiaddr(`/ip4/172.32.0.2/tcp/1`)) == true,
      `Should ignore public addresses when checking private subnets`
    )
  })
})
