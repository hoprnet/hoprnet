import heap from 'heap-js'
import { randomSubset } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import debug from 'debug'
const log = debug('hopr-core:network-peers')

type Entry = {
  id: PeerId
  heartbeatsSent: number
  heartbeatsSuccess: number
  lastSeen: number
  lastPingSuccess: boolean
}

class NetworkPeers {
  private peers: Entry[]

  private compareLastPing(a: Entry, b: Entry) {
    return a.lastSeen - b.lastSeen
  }

  private find(peer: PeerId): Entry | undefined {
    return this.peers.find((x) => x.id.toB58String() === peer.toB58String())
  }

  constructor(existingPeers: Array<PeerId>) {
    this.peers = []

    for (const peer of existingPeers) {
      this.register(peer)
    }
    heap.heapify(this.peers, this.compareLastPing)
  }

  public async pingOldest(interaction: (PeerID) => Promise<boolean>): Promise<void> {
    const entry = heap.heappop(this.peers, this.compareLastPing)
    if (!entry) {
      return Promise.resolve()
    }
    entry.heartbeatsSent++
    entry.lastSeen = Date.now()
    heap.heappush(this.peers, entry, this.compareLastPing)
    entry.lastPingSuccess = await interaction(entry.id)
    if (entry.lastPingSuccess) {
      entry.heartbeatsSuccess++
    }
  }

  // Get a random sample peers.
  public randomSubset(size: number, filter?: (peer: PeerId) => boolean): PeerId[] {
    return randomSubset(
      this.peers,
      Math.min(size, this.peers.length),
      filter != null ? (entry: Entry) => filter(entry.id) : undefined
    ).map((e: Entry) => e.id)
  }

  public register(id: PeerId) {
    if (!this.find(id)) {
      heap.heappush(
        this.peers,
        { id, heartbeatsSent: 0, heartbeatsSuccess: 0, lastSeen: Date.now(), lastPingSuccess: true },
        this.compareLastPing
      )
    }
  }

  public length(): number {
    return this.peers.length
  }

  public all(): PeerId[] {
    return this.peers.map((x) => x.id)
  }

  public has(peer: PeerId): boolean {
    return this.peers.findIndex((entry: Entry) => entry.id.equals(peer)) >= 0
  }

  public debugLog() {
    log(`current nodes:`)
    this.peers.forEach((node: Entry) => log(node.id.toB58String()))
  }

  public containsOlderThan(ts): boolean {
    return this.peers.length > 0 && heap.heaptop(this.peers, 1, this.compareLastPing)[0].lastSeen < ts
  }

  public lastSeen(peer: PeerId): number {
    return this.find(peer).lastSeen
  }

  // @returns a float between 0 (completely unreliable) and 1 (completely
  // reliable) estimating the quality of service of a peer's network connection
  public qualityOf(peer: PeerId): number {
    let entry = this.find(peer)
    if (entry && entry.heartbeatsSent > 0) {
      return entry.heartbeatsSuccess / entry.heartbeatsSent
    }
    return 0.2 // Unknown
  }
}

export default NetworkPeers
