import { Multiaddr } from '@multiformats/multiaddr'
import assert from 'assert'

import { isMultiaddrLocal } from './addressSorters.js'

describe(`test isMultiaddrLocal`, function () {
  it(`should detect local multiaddrs`, function () {
    assert(!isMultiaddrLocal(new Multiaddr('/ip4/30.0.0.1/tcp/4000')))
    assert(!isMultiaddrLocal(new Multiaddr('/ip4/31.0.0.1/tcp/4000')))
    assert(isMultiaddrLocal(new Multiaddr('/ip4/127.0.0.1/tcp/4000')))
    assert(isMultiaddrLocal(new Multiaddr('/ip6/::1/tcp/4000')))
    assert(
      isMultiaddrLocal(new Multiaddr('/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'))
    )
    assert(
      !isMultiaddrLocal(new Multiaddr('/ip4/30.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'))
    )
    assert(
      !isMultiaddrLocal(
        new Multiaddr(
          '/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb/p2p-circuit/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'
        )
      )
    )
  })
})
