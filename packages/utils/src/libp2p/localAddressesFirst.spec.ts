import { Multiaddr } from 'multiaddr'
import { expect } from 'chai'

import { localAddressesFirst } from './localAddressesFirst'

describe(`test localAddressesFirst`, function () {
    it(`should put local addresses first`, async function () {
        const addresses = [            
            {
              multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'),
              isCertified: true,
            },
            {
              multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'),
              isCertified: true,
            },
            {
              multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'),        
              isCertified: true,
            }
        ]
    
        const sortedAddresses = localAddressesFirst(addresses)
        expect(sortedAddresses).to.eql(
          [
            { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: true },
            { multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'), isCertified: true },
            { multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'), isCertified: true },
          ]
        )
    })

    it('should sort public certified addresses first', () => {
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
        expect(sortedAddresses).to.eql(
            [
                { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: true },
                { multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'), isCertified: false },
                { multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'), isCertified: true },
                { multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'), isCertified: false }
            ]
        )
    })
})
