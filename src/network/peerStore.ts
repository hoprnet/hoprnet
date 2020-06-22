import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import heap from 'heap-js'

import type PeerInfo from 'peer-info'

export type Entry = {
  id: string
  lastSeen: number
}

class PeerStore<Chain extends HoprCoreConnector> {
  peers: Entry[]

  private compare(a: Entry, b: Entry) {
    return a.lastSeen - b.lastSeen
  }
  constructor(node: Hopr<Chain>) {
    this.peers = []

    for (const peerInfo of node.peerStore.peers.values()) {
      this.peers.push({
        id: peerInfo.id.toB58String(),
        lastSeen: 0,
      })
    }

    node.on('peer:connect', (peerInfo: PeerInfo) => this.push({ id: peerInfo.id.toB58String(), lastSeen: Date.now() }))

    heap.heapify(this.peers, this.compare)
  }

  push(entry: Entry): number {
    const index = this.peers.findIndex((e: Entry) => e.id === entry.id)
    if (index >= 0) {
      this.peers[index] = entry
      heap.heapify(this.peers, this.compare)
      return
    }

    heap.heappush(this.peers, entry, this.compare)

    return this.peers.length
  }

  replace(newEntry: Entry) {
    heap.heapreplace(this.peers, newEntry, this.compare)
  }

  has(peer: string) {
    return this.peers.findIndex((entry: Entry) => entry.id === peer) >= 0
  }

  top(n: number): Entry[] {
    return heap.heaptop(this.peers, n, this.compare)
  }

  pop(): Entry {
    return heap.heappop(this.peers, this.compare)
  }
}

export default PeerStore
