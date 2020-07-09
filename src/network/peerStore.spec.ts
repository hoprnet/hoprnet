import type Hopr from '..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import assert from 'assert'

import PeerStore, { BLACKLIST_TIMEOUT } from './peerStore'
import PeerInfo from 'peer-info'

import { EventEmitter } from 'events'

function generateNode() {
  class Dummy extends EventEmitter {
    peerStore = {
      peers: new Map<string, PeerInfo>(),
    }
  }

  return (new Dummy() as unknown) as Hopr<HoprCoreConnector>
}

describe('test PeerStore', function () {
  const peerStore = new PeerStore(generateNode())
  it('should push and pop elements', function () {
    assert(peerStore.length == 0, 'peerStore must be empty')
    peerStore.push({
      id: '1',
      lastSeen: Date.now(),
    })

    peerStore.push({
      id: '2',
      lastSeen: Date.now() - 10,
    })

    assert(peerStore.top(1)[0].id === '2', `Recently seen peer should be on top.`)

    assert(peerStore.top(2)[1].id === '1', 'Less recently seen peer should be second.')

    peerStore.push({
      id: '1',
      lastSeen: Date.now() - 20,
    })

    assert(peerStore.peers.length == 2, `Updating a peer should not increase the heap size.`)

    assert(
      peerStore.top(1)[0].id === '1',
      `Updating a peer with a more recent 'lastSeen' property should change the order`
    )

    peerStore.pop()

    assert(
      peerStore.top(1)[0].id === '2',
      `After removing the most recently seen peer, the less recently seen peer should be at top.`
    )

    peerStore.pop()

    assert(peerStore.length == 0, `peerStore must be empty`)
  })

  it('should push, pop and blacklist peers', function () {
    assert(peerStore.length == 0, 'peerStore must be empty')

    peerStore.wipeBlacklist()

    peerStore.blacklistPeer('1')

    assert(peerStore.deletedPeers.length == 1, `blacklist must contain the just blacklisted node`)

    peerStore.push({
      id: '1',
      lastSeen: Date.now(),
    })

    assert(peerStore.length == 0, `adding a blacklisted peers must not change the peerStore`)

    peerStore.wipeBlacklist()

    // @ts-ignore
    assert(peerStore.deletedPeers.length == 0, `blacklist must be empty now`)

    peerStore.push({
      id: '1',
      lastSeen: Date.now(),
    })

    peerStore.blacklistPeer('1')

    // @ts-ignore
    assert(
      peerStore.length == 0 && peerStore.deletedPeers.length == 1,
      `peer must have been removed from peerStore and added to deletedPeers`
    )

    peerStore.blacklistPeer('1')

    assert(peerStore.deletedPeers.length == 1, `peer must not be added twice to the blacklist`)

    peerStore.blacklistPeer('2')

    peerStore.deletedPeers[0].deletedAt -= BLACKLIST_TIMEOUT + 1

    peerStore.blacklistPeer('3')

    // @ts-ignore
    assert(peerStore.deletedPeers.length == 2, `the cleanup process should have removed the first node`)

    peerStore.wipeBlacklist()
    assert(
      // @ts-ignore
      peerStore.deletedPeers.length == 0 && peerStore.peers.length == 0,
      `blacklist must be empty and there must be no nodes in the peerStore`
    )
  })
})
