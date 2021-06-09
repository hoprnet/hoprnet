import { Multiaddr } from 'multiaddr'
import { expect } from 'chai'

import { localAddressesFirst, publicAddressesFirst, isMultiaddrPrivate } from './addressSorters'

describe(`test isMultiaddrPrivate`, function () {
  it(`should detect private multiaddrs`, function () {
    expect(isMultiaddrPrivate(new Multiaddr('/ip4/30.0.0.1/tcp/4000'))).to.eql(false)
    expect(isMultiaddrPrivate(new Multiaddr('/ip4/31.0.0.1/tcp/4000'))).to.eql(false)
    expect(isMultiaddrPrivate(new Multiaddr('/ip4/127.0.0.1/tcp/4000'))).to.eql(true)
    expect(isMultiaddrPrivate(new Multiaddr('/ip6/::1/tcp/4000'))).to.eql(true)
    expect(isMultiaddrPrivate(new Multiaddr('/p2p-circuit/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N'))).to.eql(
      false
    )
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

describe(`test publicAddressesFirst`, function () {
  it(`should put public addresses first`, async function () {
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

    const sortedAddresses = publicAddressesFirst(addresses)
    expect(sortedAddresses).to.eql([
      { multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: true }
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

    const sortedAddresses = publicAddressesFirst(addresses)
    expect(sortedAddresses).to.eql([
      { multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'), isCertified: false },
      { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: true },
      { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: false }
    ])
  })
})
