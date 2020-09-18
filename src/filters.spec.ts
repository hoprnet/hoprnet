import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import { peerHasOnlyPublicAddresses, peerHasOnlyPrivateAddresses } from './filters'
import assert from 'assert'
import Multiaddr from 'multiaddr'

describe('filters', () => {
  it('peers', async () => {
    const unConnectedPeer = await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' }))
    assert(peerHasOnlyPublicAddresses(unConnectedPeer) == false)
    assert(peerHasOnlyPrivateAddresses(unConnectedPeer) == false)

    const privatePeer = await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' }))
    privatePeer.multiaddrs.add(new Multiaddr('/ip4/127.0.0.1/tcp/9090'))
    assert(peerHasOnlyPublicAddresses(privatePeer) == false)
    assert(peerHasOnlyPrivateAddresses(privatePeer) == true)

    const publicPeer = await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' }))
    publicPeer.multiaddrs.add(new Multiaddr('/ip4/123.4.56.7/tcp/9090'))
    assert(peerHasOnlyPublicAddresses(publicPeer) == true)
    assert(peerHasOnlyPrivateAddresses(publicPeer) == false)

    const mixedPeer = await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' }))
    mixedPeer.multiaddrs.add(new Multiaddr('/ip4/123.4.56.7/tcp/9090'))
    mixedPeer.multiaddrs.add(new Multiaddr('/ip4/127.0.0.1/tcp/9090'))
    assert(peerHasOnlyPublicAddresses(mixedPeer) == false)
    assert(peerHasOnlyPrivateAddresses(mixedPeer) == false)
  })
})
