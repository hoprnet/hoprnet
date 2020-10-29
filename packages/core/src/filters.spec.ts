import {peerHasOnlyPublicAddresses, peerHasOnlyPrivateAddresses} from './filters'
import assert from 'assert'
import Multiaddr from 'multiaddr'

describe('filters', () => {
  it('peers', async () => {
    const unConnectedPeer = []
    assert(peerHasOnlyPublicAddresses(unConnectedPeer) == false)
    assert(peerHasOnlyPrivateAddresses(unConnectedPeer) == false)

    const privatePeer = [new Multiaddr('/ip4/127.0.0.1/tcp/9090')]
    assert(peerHasOnlyPublicAddresses(privatePeer) == false)
    assert(peerHasOnlyPrivateAddresses(privatePeer) == true)

    privatePeer.push(new Multiaddr('/ip4/0.0.0.0/tcp/9093'))
    assert(peerHasOnlyPublicAddresses(privatePeer) == false)
    assert(peerHasOnlyPrivateAddresses(privatePeer) == true)

    const publicPeer = [new Multiaddr('/ip4/123.4.56.7/tcp/9090')]
    assert(peerHasOnlyPublicAddresses(publicPeer) == true)
    assert(peerHasOnlyPrivateAddresses(publicPeer) == false)

    const mixedPeer = [new Multiaddr('/ip4/123.4.56.7/tcp/9090'), new Multiaddr('/ip4/127.0.0.1/tcp/9090')]
    assert(peerHasOnlyPublicAddresses(mixedPeer) == false)
    assert(peerHasOnlyPrivateAddresses(mixedPeer) == false)
  })
})
