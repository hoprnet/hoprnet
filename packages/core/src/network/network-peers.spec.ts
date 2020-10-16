import assert from 'assert'

import PeerStore, { BLACKLIST_TIMEOUT } from './network-peers'

describe('test PeerStore', function () {
  const empty = [][Symbol.iterator]()
  const networkPeers = new PeerStore(empty)
  it('should push and pop elements', function () {
    assert(networkPeers.length == 0, 'networkPeers must be empty')
    networkPeers.push({
      id: '1',
      lastSeen: Date.now()
    })

    networkPeers.push({
      id: '2',
      lastSeen: Date.now() - 10
    })

    assert(networkPeers.top(1)[0].id === '2', `Recently seen peer should be on top.`)

    assert(networkPeers.top(2)[1].id === '1', 'Less recently seen peer should be second.')

    networkPeers.push({
      id: '1',
      lastSeen: Date.now() - 20
    })

    assert(networkPeers.peers.length == 2, `Updating a peer should not increase the heap size.`)

    assert(
      networkPeers.top(1)[0].id === '1',
      `Updating a peer with a more recent 'lastSeen' property should change the order`
    )

    networkPeers.pop()

    assert(
      networkPeers.top(1)[0].id === '2',
      `After removing the most recently seen peer, the less recently seen peer should be at top.`
    )

    networkPeers.pop()

    assert(networkPeers.length == 0, `networkPeers must be empty`)
  })

  it('should push, pop and blacklist peers', function () {
    assert(networkPeers.length == 0, 'networkPeers must be empty')

    networkPeers.wipeBlacklist()

    networkPeers.blacklistPeer('1')

    assert(networkPeers.deletedPeers.length == 1, `blacklist must contain the just blacklisted node`)

    networkPeers.push({
      id: '1',
      lastSeen: Date.now()
    })

    assert(networkPeers.length == 0, `adding a blacklisted peers must not change the networkPeers`)

    networkPeers.wipeBlacklist()

    // @ts-ignore
    assert(networkPeers.deletedPeers.length == 0, `blacklist must be empty now`)

    networkPeers.push({
      id: '1',
      lastSeen: Date.now()
    })

    networkPeers.blacklistPeer('1')

    // @ts-ignore
    assert(
      networkPeers.length == 0 && networkPeers.deletedPeers.length == 1,
      `peer must have been removed from networkPeers and added to deletedPeers`
    )

    networkPeers.blacklistPeer('1')

    assert(networkPeers.deletedPeers.length == 1, `peer must not be added twice to the blacklist`)

    networkPeers.blacklistPeer('2')

    networkPeers.deletedPeers[0].deletedAt -= BLACKLIST_TIMEOUT + 1

    networkPeers.blacklistPeer('3')

    // @ts-ignore
    assert(networkPeers.deletedPeers.length == 2, `the cleanup process should have removed the first node`)

    networkPeers.wipeBlacklist()
    assert(
      // @ts-ignore
      networkPeers.deletedPeers.length == 0 && networkPeers.peers.length == 0,
      `blacklist must be empty and there must be no nodes in the networkPeers`
    )
  })
})
