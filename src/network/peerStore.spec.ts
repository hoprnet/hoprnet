import type Hopr from '..'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import assert from 'assert'

import PeerStore from './peerStore'
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
  })
})
