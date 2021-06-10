import { Multiaddr } from 'multiaddr'
import { expect } from 'chai'

import { localAddressesFirst, isMultiaddrLocal } from './addressSorters'

describe(`test isMultiaddrLocal`, function () {
  it(`should detect local multiaddrs`, function () {
    expect(isMultiaddrLocal(new Multiaddr('/ip4/30.0.0.1/tcp/4000'))).to.eql(false)
    expect(isMultiaddrLocal(new Multiaddr('/ip4/31.0.0.1/tcp/4000'))).to.eql(false)
    expect(isMultiaddrLocal(new Multiaddr('/ip4/127.0.0.1/tcp/4000'))).to.eql(true)
    expect(isMultiaddrLocal(new Multiaddr('/ip6/::1/tcp/4000'))).to.eql(true)
    expect(
      isMultiaddrLocal(new Multiaddr('/ip4/127.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'))
    ).to.eql(true)
    expect(
      isMultiaddrLocal(new Multiaddr('/ip4/30.0.0.1/tcp/0/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'))
    ).to.eql(false)
    expect(
      isMultiaddrLocal(
        new Multiaddr(
          '/p2p/16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb/p2p-circuit/p2p/16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg'
        )
      )
    ).to.eql(false)
  })
})

describe(`test localAddressesFirst`, function () {
  it(`should put local addresses first`, async function () {
    const addresses = [
      {
        multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'),
        isCertified: true
      },
      {
        multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'),
        isCertified: true
      },
      {
        multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'),
        isCertified: true
      }
    ]

    const sortedAddresses = localAddressesFirst(addresses)
    expect(sortedAddresses).to.eql([
      { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'), isCertified: true }
    ])
  })

  it('should put certified addresses first', () => {
    const addresses = [
      {
        multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'),
        isCertified: false
      },
      {
        multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'),
        isCertified: true
      },
      {
        multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'),
        isCertified: false
      },
      {
        multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'),
        isCertified: true
      }
    ]

    const sortedAddresses = localAddressesFirst(addresses)
    expect(sortedAddresses).to.eql([
      { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: false },
      { multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'), isCertified: false }
    ])
  })
})
