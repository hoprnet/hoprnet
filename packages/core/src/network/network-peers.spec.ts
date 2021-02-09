import assert from 'assert'
import { NETWORK_QUALITY_THRESHOLD as Q } from '../constants'
import { fakePeerId } from '../test-utils'
import PeerStore from './network-peers'

describe('test PeerStore', async function () {
  const IDS = [fakePeerId(1), fakePeerId(2), fakePeerId(3), fakePeerId(4)]

  const SELF = fakePeerId(6)
  it('should register new peers', async function () {
    const networkPeers = new PeerStore([], [SELF])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')

    networkPeers.register(SELF)
    assert(networkPeers.length() == 0, 'networkPeers must be empty after inserting self')

    networkPeers.register(IDS[0])
    networkPeers.register(IDS[1])
    assert(networkPeers.length() == 2, 'now has 2 peers')
    networkPeers.register(IDS[0])
    assert(networkPeers.length() == 2, `Updating a peer should not increase len`)
  })

  it('should allow randomSubset to be taken of peer ids', function () {
    const networkPeers = new PeerStore(IDS, [SELF])
    assert(networkPeers.randomSubset(3).length == 3)
  })

  it('should _ping_ peers', async function () {
    const id = fakePeerId(5)
    const networkPeers = new PeerStore([], [SELF])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')
    assert(networkPeers.pingSince(123).length === 0, 'no peers yet')

    networkPeers.register(id)
    assert(networkPeers.qualityOf(id) < Q, 'initial peers have low quality')
    assert(networkPeers.length() === 1)

    await networkPeers.ping(id, () => Promise.resolve(true)) // 0.3
    await networkPeers.ping(id, () => Promise.resolve(true)) // 0.4
    await networkPeers.ping(id, () => Promise.resolve(true)) // 0.5
    await networkPeers.ping(id, () => Promise.resolve(true)) // 0.6
    assert(networkPeers.qualityOf(id) > Q, 'after 4 successful ping, peer is good quality')

    await networkPeers.ping(id, () => Promise.resolve(false)) //0.5
    assert(networkPeers.qualityOf(id) <= Q, 'after 1 failed pings, peer is bad quality')

    await networkPeers.ping(id, () => Promise.resolve(true))
    await networkPeers.ping(id, () => Promise.resolve(true))
    assert(networkPeers.qualityOf(id) > Q, 'after 2 more pings, peer is good again')
  })
})
