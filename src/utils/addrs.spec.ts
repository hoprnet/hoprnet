import assert from 'assert'
import { parseAddress } from './addrs'
import { Multiaddr } from 'multiaddr'

describe('test address parsing', function () {
  it('good examples', function () {
    assert(parseAddress(new Multiaddr('/ip4/127.0.0.1/tcp/0')).valid)

    assert(
      parseAddress(new Multiaddr('/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'))
        .valid
    )

    assert(parseAddress(new Multiaddr('/ip6/::1/tcp/0')).valid)

    assert(
      parseAddress(new Multiaddr('/ip6/::1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg')).valid
    )

    assert(
      parseAddress(
        new Multiaddr(
          '/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb/p2p-circuit/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'
        )
      ).valid
    )
  })

  it('bad examples', function () {
    assert(!parseAddress(new Multiaddr('/tcp/0')).valid)

    assert(!parseAddress(new Multiaddr('/ip4/127.0.0.1/udp/0')).valid)

    assert(!parseAddress(new Multiaddr('/ip4/127.0.0.1')).valid)

    assert(!parseAddress(new Multiaddr('/ip6/::1/udp/0')).valid)

    assert(!parseAddress(new Multiaddr('/ip6/::1')).valid)

    assert(
      !parseAddress(
        new Multiaddr(
          '/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb/p2p-circuit/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb'
        )
      ).valid
    )

    assert(!parseAddress(new Multiaddr('/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb')).valid)

    assert(!parseAddress(new Multiaddr('/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb/tcp/0')).valid)
  })
})
