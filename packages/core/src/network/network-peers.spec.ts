import assert from 'assert'
import { BLACKLIST_TIMEOUT } from '../constants'
import PeerId from 'peer-id'

import PeerStore from './network-peers'

const IDS = ([
  '16Uiu2HAmDaiXdDZbcFvCXd1ZvaxnrXbLNSEt8jH7iEFqv4A3suZU',
  '16Uiu2HAmKae7qFncLFydcqCCqTXVFojtafVktXpSRSRZyDqZpkbP',
  '16Uiu2HAmHxG2fj7MUNzopRy3SKLfeEHgzbBQJp6tc8WGkLmVddBE',
  '16Uiu2HAm4cvqqFbFnYU5jFT1HKvA7N2GZYx2XKUec5hFSeJRDgL9'
]).map(x => PeerId.createFromB58String(x))

describe('test PeerStore', function () {

  const empty = [][Symbol.iterator]()
  const networkPeers = new PeerStore(empty)

  it('should push and pop elements', function () {

    assert(networkPeers.length == 0, 'networkPeers must be empty')
    networkPeers.push({
      id: IDS[0],
      lastSeen: Date.now()
    })

    networkPeers.push({
      id: IDS[1],
      lastSeen: Date.now() - 10
    })

    assert(networkPeers.top(1)[0].id === IDS[1], `Recently seen peer should be on top.`)
    assert(networkPeers.top(2)[1].id === IDS[0], 'Less recently seen peer should be second.')

    networkPeers.push({
      id: IDS[0],
      lastSeen: Date.now() - 20
    })

    assert(networkPeers.peers.length == 2, `Updating a peer should not increase the heap size.`)
    assert(
      networkPeers.top(1)[0].id === IDS[0],
      `Updating a peer with a more recent 'lastSeen' property should change the order`
    )

    networkPeers.pop()
    assert(
      networkPeers.top(1)[0].id === IDS[1],
      `After removing the most recently seen peer, the less recently seen peer should be at top.`
    )

    networkPeers.reset()
  })

  it('should allow randomSubset to be taken of peer ids', function() {

    IDS.forEach(id => {
      networkPeers.push({
        id,
        lastSeen: Date.now()
      })
    })


    networkPeers.randomSubset(3)



    networkPeers.reset()
  })

  it('should push, pop and blacklist peers', function () {
    assert(networkPeers.length == 0, 'networkPeers must be empty')

    networkPeers.wipeBlacklist()
    networkPeers.blacklistPeer(IDS[0])

    assert(networkPeers.deletedPeers.length == 1, `blacklist must contain the just blacklisted node`)

    networkPeers.push({
      id:  IDS[0],
      lastSeen: Date.now()
    })

    assert(networkPeers.length == 0, `adding a blacklisted peers must not change the networkPeers`)

    networkPeers.wipeBlacklist()

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
  })
})
