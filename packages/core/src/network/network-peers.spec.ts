import assert from 'assert'
import { fakePeerId, showBackoff } from '../test-utils.spec.js'
import NetworkPeers, { MAX_BACKOFF, NetworkPeersOrigin } from './network-peers.js'

const NETWORK_QUALITY_THRESHOLD = 0.5

describe('test PeerStore', async function () {
  const IDS = [fakePeerId(1), fakePeerId(2), fakePeerId(3), fakePeerId(4)]

  const SELF = fakePeerId(6)
  it('should register new peers', async function () {
    const networkPeers = new NetworkPeers([], [SELF], NETWORK_QUALITY_THRESHOLD)
    assert(networkPeers.length() == 0, 'networkPeers must be empty')

    networkPeers.register(SELF, NetworkPeersOrigin.TESTING)
    assert(networkPeers.length() == 0, 'networkPeers must be empty after inserting self')

    networkPeers.register(IDS[0], NetworkPeersOrigin.TESTING)
    networkPeers.register(IDS[1], NetworkPeersOrigin.TESTING)
    assert(networkPeers.length() == 2, 'now has 2 peers')
    networkPeers.register(IDS[0], NetworkPeersOrigin.TESTING)
    assert(networkPeers.length() == 2, `Updating a peer should not increase len`)
  })

  it('should allow randomSubset to be taken of peer ids', function () {
    const networkPeers = new NetworkPeers(IDS, [SELF], NETWORK_QUALITY_THRESHOLD)
    assert(networkPeers.randomSubset(3).length == 3)
  })

  it('should _ping_ peers', async function () {
    const id = fakePeerId(5)
    const networkPeers = new NetworkPeers([], [SELF], NETWORK_QUALITY_THRESHOLD)
    assert(networkPeers.length() == 0, 'networkPeers must be empty')
    assert(networkPeers.pingSince(123).length === 0, 'no peers yet')

    networkPeers.register(id, NetworkPeersOrigin.TESTING)
    assert(networkPeers.qualityOf(id) < NETWORK_QUALITY_THRESHOLD, 'initial peers have low quality')
    assert(networkPeers.length() === 1)

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // NETWORK_QUALITY_THRESHOLD
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
    assert(networkPeers.qualityOf(id) > NETWORK_QUALITY_THRESHOLD, 'after 4 successful ping, peer is good quality')

    networkPeers.updateRecord({
      destination: id,
      lastSeen: -1
    }) // 0.5
    assert(networkPeers.qualityOf(id) <= NETWORK_QUALITY_THRESHOLD, 'after 1 failed pings, peer is bad quality')

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.5

    networkPeers.updateRecord({
      destination: id,
      lastSeen: Date.now()
    }) // 0.6
    assert(networkPeers.qualityOf(id) > NETWORK_QUALITY_THRESHOLD, 'after 2 more pings, peer is good again')
  })

  it('should detect that node is offline', async function () {
    let peerConsideredOffline = false

    const onPeerOffline = () => {
      peerConsideredOffline = true
    }

    const networkPeers = new NetworkPeers([], [SELF], NETWORK_QUALITY_THRESHOLD, onPeerOffline)

    const id = fakePeerId(5)
    networkPeers.register(id, NetworkPeersOrigin.TESTING)

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
