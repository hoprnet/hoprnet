import { Multiaddr } from 'multiaddr'
import { expect } from 'chai'

import { localAddressesFirst } from './localAddressesFirst'

describe(`test localAddressesFirst`, function () {
    it(`should sort addresses, local first`, async function () {
        const addresses = [            
            {
              multiaddr: new Multiaddr('/ip4/30.0.0.1/tcp/4000'),
            },
            {
              multiaddr: new Multiaddr('/ip4/127.0.0.1/tcp/4000'),
            },
            {
              multiaddr: new Multiaddr('/ip4/31.0.0.1/tcp/4000'),        
            }
        ]
    
        const sortedAddresses = localAddressesFirst(addresses)
        expect(sortedAddresses[0].multiaddr.equals(new Multiaddr('/ip4/30.0.0.1/tcp/4000'))).to.eql(true)
        expect(sortedAddresses[1].multiaddr.equals(new Multiaddr('/ip4/31.0.0.1/tcp/4000'))).to.eql(true)
        expect(sortedAddresses[2].multiaddr.equals(new Multiaddr('/ip4/127.0.0.1/tcp/4000'))).to.eql(true)        })
})
