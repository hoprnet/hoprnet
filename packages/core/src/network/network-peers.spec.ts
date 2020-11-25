import assert from 'assert'
import { NETWORK_QUALITY_THRESHOLD as Q } from '../constants'
import { fakePeerId } from '../test-utils'
import PeerStore from './network-peers'

describe('test PeerStore', async function () {
  const IDS = [fakePeerId(1), fakePeerId(2), fakePeerId(3), fakePeerId(4)]

  it('should register new peers', function () {
    const networkPeers = new PeerStore([])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')
    networkPeers.register(IDS[0])
    networkPeers.register(IDS[1])
    assert(networkPeers.length() == 2, 'now has 2 peers')
    networkPeers.register(IDS[0])
    assert(networkPeers.length() == 2, `Updating a peer should not increase len`)
  })

  it('should allow randomSubset to be taken of peer ids', function () {
    const networkPeers = new PeerStore(IDS)
    assert(networkPeers.randomSubset(3).length == 3)
  })

  it('should _ping_ peers', async function () {
    const id = fakePeerId(5)
    const networkPeers = new PeerStore([])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')
    await networkPeers.pingOldest(() => {
      throw new Error('Empty networkPeers in ping')
    })
    networkPeers.register(id)
    assert(networkPeers.qualityOf(id) < Q, 'initial peers have low quality')
    assert(networkPeers.length() === 1)

    await networkPeers.pingOldest(() => Promise.resolve(true))
    assert(networkPeers.qualityOf(id) > Q, 'after first successful ping, peer is good quality')
    await networkPeers.pingOldest(() => Promise.resolve(false))
    assert(networkPeers.qualityOf(id) <= Q, 'after 50% failed pings, peer is bad quality')

    await networkPeers.pingOldest(() => Promise.resolve(true))
    await networkPeers.pingOldest(() => Promise.resolve(true))
    assert(networkPeers.qualityOf(id) > Q, 'after 25% failed pings, peer is good again')
  })
})
