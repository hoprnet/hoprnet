import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import heap from 'heap-js'

import type PeerInfo from 'peer-info'
import { durations } from '@hoprnet/hopr-utils'

import debug from 'debug'
const log = debug('hopr-core:peerStore')

export type Entry = {
  id: string
  lastSeen: number
}

export type BlacklistedEntry = {
  id: string
  deletedAt: number
}

export const BLACKLIST_TIMEOUT = durations.seconds(47)

class PeerStore<Chain extends HoprCoreConnector> {
  peers: Entry[]

  deletedPeers: BlacklistedEntry[]

  private compare(a: Entry, b: Entry) {
    return a.lastSeen - b.lastSeen
  }

  private compareBlackList(a: BlacklistedEntry, b: BlacklistedEntry) {
    return b.deletedAt - a.deletedAt
  }

  constructor(node: Hopr<Chain>) {
    this.peers = []
    this.deletedPeers = []

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
    const THRESHOLD_TIMEOUT = Date.now() - BLACKLIST_TIMEOUT
    this.cleanupBlacklist(THRESHOLD_TIMEOUT)

    const blacklistIndex = this.deletedPeers.findIndex((e: BlacklistedEntry) => e.id === entry.id)

    if (blacklistIndex >= 0) {
      log(
        `Not adding peer ${entry.id} because it got blacklisted at ${new Date(
          this.deletedPeers[blacklistIndex].deletedAt
        ).toString()}`
      )
      return this.peers.length
    }

    const index = this.peers.findIndex((e: Entry) => e.id === entry.id)
    if (index >= 0) {
      this.peers[index] = entry
      heap.heapify(this.peers, this.compare)
      return this.peers.length
    }

    heap.heappush(this.peers, entry, this.compare)

    return this.peers.length
  }

  replace(newEntry: Entry) {
    heap.heapreplace(this.peers, newEntry, this.compare)
  }

  has(peer: string): boolean {
    return this.peers.findIndex((entry: Entry) => entry.id === peer) >= 0
  }

  top(n: number): Entry[] {
    return heap.heaptop(this.peers, n, this.compare)
  }

  pop(): Entry {
    return heap.heappop(this.peers, this.compare)
  }

  blacklistPeer(peer: string): number {
    const entry = {
      id: peer,
      deletedAt: Date.now(),
    }

    // (Efficiently) pushes peer information into blacklist
    const THRESHOLD_TIMEOUT = Date.now() - BLACKLIST_TIMEOUT
    const blacklistIndex = this.deletedPeers.findIndex((e: BlacklistedEntry) => e.id === peer)
    if (blacklistIndex >= 0) {
      this.deletedPeers[blacklistIndex] = entry
      heap.heapify(this.deletedPeers, this.compareBlackList)
    } else if (this.deletedPeers.length == 0) {
      this.deletedPeers.push(entry)
    } else if (this.deletedPeers.length > 0) {
      if (heap.heaptop(this.deletedPeers, 1)[0].deletedAt > THRESHOLD_TIMEOUT) {
        heap.heapreplace(this.deletedPeers, entry, this.compareBlackList)
      } else {
        heap.heappush(this.deletedPeers, entry, this.compareBlackList)
      }
    }

    // Keeps the blacklist up-to-date
    this.cleanupBlacklist(THRESHOLD_TIMEOUT)

    // Removes the peer information from our peerstore
    const index = this.peers.findIndex((e: Entry) => e.id === entry.id)
    if (index >= 0) {
      if (index == this.peers.length - 1) {
        this.peers.pop()
      } else {
        this.peers[index] = this.peers.pop()
      }
      heap.heapify(this.peers, this.compare)
    }

    return this.deletedPeers.length
  }

  wipeBlacklist(): void {
    this.deletedPeers = []
  }

  get length(): number {
    return this.peers.length
  }

  cleanupBlacklist(THRESHOLD_TIMEOUT: number = Date.now() - BLACKLIST_TIMEOUT) {
    while (
      this.deletedPeers.length > 0 &&
      heap.heaptop(this.deletedPeers, 1, this.compareBlackList)[0].deletedAt < THRESHOLD_TIMEOUT
    ) {
      heap.heappop(this.deletedPeers, this.compareBlackList)
    }
  }
}

export default PeerStore
