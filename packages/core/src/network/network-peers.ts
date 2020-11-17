import heap from 'heap-js'
import { randomSubset } from '@hoprnet/hopr-utils'

import PeerId from 'peer-id'
import { BLACKLIST_TIMEOUT } from '../constants'

import debug from 'debug'
const log = debug('hopr-core:network-peers')
const verbose = debug('hopr-core:verbose:network-peers')

export type Entry = {
  id: PeerId
  lastSeen: number
}

export type BlacklistedEntry = {
  id: PeerId
  deletedAt: number
}

class NetworkPeers {
  peers: Entry[]

  deletedPeers: BlacklistedEntry[]

  private compare(a: Entry, b: Entry) {
    return a.lastSeen - b.lastSeen
  }

  private compareBlackList(a: BlacklistedEntry, b: BlacklistedEntry) {
    return b.deletedAt - a.deletedAt
  }

  constructor(existingPeers: Array<PeerId>) {
    this.peers = []
    this.deletedPeers = []

    for (const peer of existingPeers) {
      this.peers.push({
        id: peer,
        lastSeen: 0
      })
    }

    heap.heapify(this.peers, this.compare)
  }

  // Get a random sample of non-blacklisted peers.
  randomSubset(size: number, filter?: (peer: PeerId) => boolean): PeerId[] {
    return randomSubset(
      this.peers,
      Math.min(size, this.peers.length),
      filter != null ? (entry: Entry) => filter(entry.id) : undefined
    ).map((e: Entry) => e.id)
  }

  onPeerConnect(peerId: PeerId) {
    this.push({ id: peerId, lastSeen: Date.now() })
  }

  push(entry: Entry): number {
    verbose('adding', entry.id.toB58String())
    const THRESHOLD_TIMEOUT = Date.now() - BLACKLIST_TIMEOUT
    this.cleanupBlacklist(THRESHOLD_TIMEOUT)

    const blacklistIndex = this.deletedPeers.findIndex((e: BlacklistedEntry) => e.id.equals(entry.id))

    if (blacklistIndex >= 0) {
      log(
        `Not adding peer ${entry.id.toB58String()} because it got blacklisted at ${new Date(
          this.deletedPeers[blacklistIndex].deletedAt
        ).toString()}`
      )
      return this.peers.length
    }

    const index = this.peers.findIndex((e: Entry) => e.id.equals(entry.id))
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

  has(peer: PeerId): boolean {
    return this.peers.findIndex((entry: Entry) => entry.id.equals(peer)) >= 0
  }

  hasBlacklisted(peer: PeerId): boolean {
    return this.deletedPeers.findIndex((entry: BlacklistedEntry) => entry.id.equals(peer)) >= 0
  }

  top(n: number): Entry[] {
    return heap.heaptop(this.peers, n, this.compare)
  }

  pop(): Entry {
    return heap.heappop(this.peers, this.compare)
  }

  blacklistPeer(peer: PeerId): number {
    verbose('blacklisting', peer.toB58String())
    const entry = {
      id: peer,
      deletedAt: Date.now()
    }

    // (Efficiently) pushes peer information into blacklist
    const THRESHOLD_TIMEOUT = Date.now() - BLACKLIST_TIMEOUT
    const blacklistIndex = this.deletedPeers.findIndex((e: BlacklistedEntry) => e.id.equals(peer))
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
    const index = this.peers.findIndex((e: Entry) => e.id.equals(entry.id))
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
    verbose('wiping blacklist')
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

  public debugLog() {
    log(`current nodes:`)
    this.peers.forEach((node: Entry) => log(node.id.toB58String()))
  }

  updatedSince(ts) {
    return this.peers.length > 0 && this.top(1)[0].lastSeen < ts
  }

  reset() {
    this.peers = []
  }

  // @returns a float between 0 (completely unreliable) and 1 (completely
  // reliable) estimating the quality of service of a peer's network connection
  public qualityOf(peer: PeerId): number {
    // TODO replace this
    for (let entry of this.peers) {
      if (entry.id.equals(peer)) {
        return 1
      }
    }
    for (let entry of this.deletedPeers) {
      if (entry.id.equals(peer)) {
        return 0
      }
    }
    return 0.2 // Unknown
  }
}

export default NetworkPeers
