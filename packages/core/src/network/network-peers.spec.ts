import assert from 'assert'
import { NETWORK_QUALITY_THRESHOLD, NETWORK_QUALITY_THRESHOLD as Q } from '../constants'
import { fakePeerId, showBackoff } from '../test-utils.spec'
import PeerStore, { MAX_BACKOFF } from './network-peers'

describe('test PeerStore', async function () {
  const IDS = [fakePeerId(1), fakePeerId(2), fakePeerId(3), fakePeerId(4)]

  const SELF = fakePeerId(6)
  it('should register new peers', async function () {
    const networkPeers = new PeerStore([], [SELF])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')

    networkPeers.register(SELF, 'test')
    assert(networkPeers.length() == 0, 'networkPeers must be empty after inserting self')

    networkPeers.register(IDS[0], 'test')
    networkPeers.register(IDS[1], 'test')
    assert(networkPeers.length() == 2, 'now has 2 peers')
    networkPeers.register(IDS[0], 'test')
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

    networkPeers.register(id, 'test')
    assert(networkPeers.qualityOf(id) < Q, 'initial peers have low quality')
    assert(networkPeers.length() === 1)

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.3
    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.4
    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.5
    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.6
    assert(networkPeers.qualityOf(id) > Q, 'after 4 successful ping, peer is good quality')

    networkPeers.updateRecord({
      destination: id,
      lastSeen: -1
    }) // 0.5
    assert(networkPeers.qualityOf(id) <= Q, 'after 1 failed pings, peer is bad quality')

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.5

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.6
    assert(networkPeers.qualityOf(id) > Q, 'after 2 more pings, peer is good again')
  })

  it('should detect that node is offline', async function () {
    let peerConsideredOffline = false

    const onPeerOffline = () => {
      peerConsideredOffline = true
    }

    const networkPeers = new PeerStore([], [SELF], onPeerOffline)

    const id = fakePeerId(5)
    networkPeers.register(id, 'test')

    while (networkPeers.qualityOf(id) <= NETWORK_QUALITY_THRESHOLD) {
      networkPeers.updateRecord({
        destination: id,
        lastSeen: Date.now()
      })
    }

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    })

    while (networkPeers.qualityOf(id) >= NETWORK_QUALITY_THRESHOLD) {
      networkPeers.updateRecord({
        destination: id,
        lastSeen: -1
      })
    }

    assert(peerConsideredOffline, 'peer should be considered offline since quality fell below threshold')
    assert(showBackoff(networkPeers) < MAX_BACKOFF, 'even offline, backoff does not reach max')
  })
})
