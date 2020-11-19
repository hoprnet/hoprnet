import assert from 'assert'
import PeerId from 'peer-id'

import PeerStore from './network-peers'

const IDS = [
  '16Uiu2HAmDaiXdDZbcFvCXd1ZvaxnrXbLNSEt8jH7iEFqv4A3suZU',
  '16Uiu2HAmKae7qFncLFydcqCCqTXVFojtafVktXpSRSRZyDqZpkbP',
  '16Uiu2HAmHxG2fj7MUNzopRy3SKLfeEHgzbBQJp6tc8WGkLmVddBE',
  '16Uiu2HAm4cvqqFbFnYU5jFT1HKvA7N2GZYx2XKUec5hFSeJRDgL9'
].map((x) => PeerId.createFromB58String(x))

describe('test PeerStore', function () {
  it('should register new peers', function () {
    const networkPeers = new PeerStore([])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')
    networkPeers.register(IDS[0])
    networkPeers.register(IDS[1])
    assert(networkPeers.length() == 2, 'now has 2 peers')
    networkPeers.register(IDS[0])
    assert(networkPeers.length() == 2, `Updating a peer should not increase the heap size.`)
  })

  it('should allow randomSubset to be taken of peer ids', function () {
    const networkPeers = new PeerStore(IDS)
    assert(networkPeers.randomSubset(3).length == 3)
  })

  it('should _ping_ peers', async function () {
    const networkPeers = new PeerStore([])
    assert(networkPeers.length() == 0, 'networkPeers must be empty')
    await networkPeers.pingOldest(() => {
      throw new Error('Empty networkPeers in ping')
    })
    networkPeers.register(IDS[0])

    /*
    //assert(networkPeers.deletedPeers.length == 0, `blacklist must be empty now`)

    networkPeers.push({
      id: IDS[0],
      lastSeen: Date.now()
    })

    networkPeers.blacklistPeer(IDS[0])

    assert(
      networkPeers.length == 0 && networkPeers.deletedPeers.length == 1,
      `peer must have been removed from networkPeers and added to deletedPeers`
    )

    networkPeers.blacklistPeer(IDS[0])
    assert(networkPeers.deletedPeers.length == 1, `peer must not be added twice to the blacklist`)

    networkPeers.blacklistPeer(IDS[1])
    networkPeers.deletedPeers[0].deletedAt -= BLACKLIST_TIMEOUT + 1
    networkPeers.blacklistPeer(IDS[2])

    //assert(networkPeers.deletedPeers.length == 2, `the cleanup process should have removed the first node`)

    networkPeers.wipeBlacklist()
    assert(
      // @ts-ignore
      networkPeers.deletedPeers.length == 0 && networkPeers.peers.length == 0,
      `blacklist must be empty and there must be no nodes in the networkPeers`
    )
    */
  })
})
