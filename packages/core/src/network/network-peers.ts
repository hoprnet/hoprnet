import { randomSubset } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { NETWORK_QUALITY_THRESHOLD } from '../constants'
import { type HeartbeatPingResult } from './heartbeat'

type Entry = {
  id: PeerId
  heartbeatsSent: number
  heartbeatsSuccess: number
  lastSeen: number
  backoff: number // between 2 and MAX_BACKOFF
  lastTen: number
}

const MIN_DELAY = 1000 // 1 sec (because this is multiplied by backoff, it will be half the actual minimum value.
const MAX_DELAY = 5 * 60 * 1000 // 5mins
const BACKOFF_EXPONENT = 1.5
export const MAX_BACKOFF = MAX_DELAY / MIN_DELAY
const UNKNOWN_Q = 0.2 // Default quality for nodes we don't know about.

class NetworkPeers {
  private peers: Entry[]

  private findIndex(peer: PeerId): number {
    return this.peers.findIndex((entry: Entry) => entry.id.equals(peer))
  }

  constructor(
    existingPeers: Array<PeerId>,
    private exclude: PeerId[] = [],
    private onPeerOffline?: (peer: PeerId) => void
  ) {
    this.peers = []

    for (const peer of existingPeers) {
      this.register(peer)
    }
  }

  private nextPing(e: Entry): number {
    // Exponential backoff
    const delay = Math.min(MAX_DELAY, Math.pow(e.backoff, BACKOFF_EXPONENT) * MIN_DELAY)
    return e.lastSeen + delay
  }

  // @returns a float between 0 (completely unreliable) and 1 (completely
  // reliable) estimating the quality of service of a peer's network connection
  public qualityOf(peer: PeerId): number {
    let entryIndex = this.findIndex(peer)
    if (entryIndex >= 0 && this.peers[entryIndex].heartbeatsSent > 0) {
      /*
      return entry.heartbeatsSuccess / entry.heartbeatsSent
      */
      return this.peers[entryIndex].lastTen
    }
    return UNKNOWN_Q
  }

  public pingSince(thresholdTime: number): PeerId[] {
    const toPing: PeerId[] = []
    for (const entry of this.peers) {
      if (this.nextPing(entry) < thresholdTime) {
        toPing.push(entry.id)
      }
    }

    return toPing
  }

  public updateRecord(pingResult: HeartbeatPingResult): void {
    const entryIndex = this.findIndex(pingResult.destination)

    if (entryIndex < 0) {
      return
    }

    const previousEntry = this.peers[entryIndex]

    let newEntry: Entry

    if (pingResult.lastSeen >= 0) {
      newEntry = {
        id: pingResult.destination,
        heartbeatsSent: previousEntry.heartbeatsSent + 1,
        lastSeen: Date.now(),
        heartbeatsSuccess: previousEntry.heartbeatsSuccess + 1,
        backoff: 2, // RESET - to back down: Math.pow(entry.backoff, 1/BACKOFF_EXPONENT)
        lastTen: Math.min(1, previousEntry.lastTen + 0.1)
      }
    } else {
      newEntry = {
        id: pingResult.destination,
        heartbeatsSent: previousEntry.heartbeatsSent + 1,
        lastSeen: Date.now(),
        heartbeatsSuccess: previousEntry.heartbeatsSuccess,
        backoff: Math.min(MAX_BACKOFF, Math.pow(previousEntry.backoff, BACKOFF_EXPONENT)),
        lastTen: Math.max(0, previousEntry.lastTen - 0.1)
      }

      if (newEntry.lastTen < NETWORK_QUALITY_THRESHOLD) {
        this.onPeerOffline?.(pingResult.destination)
      }
    }

    this.peers[entryIndex] = newEntry
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
    if (!this.has(id) && this.exclude.findIndex((x: PeerId) => id.equals(x)) < 0) {
      this.peers.push({
        id,
        heartbeatsSent: 0,
        heartbeatsSuccess: 0,
        lastSeen: Date.now(),
        backoff: 2,
        lastTen: UNKNOWN_Q
      })
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

  public debugLog(): string {
    if (this.peers.length == 0) {
      return 'no connected peers'
    }

    const peers = this.peers.map((entry) => entry.id)

    // Sort a copy of peers in-place
    peers.sort((a, b) => this.qualityOf(b) - this.qualityOf(a))

    const goodAvailabilityIndex = peers.findIndex((peer) => this.qualityOf(peer) == 1.0)

    const worstAvailabilityIndex = peers.findIndex((peer) => this.qualityOf(peer) == 0.0)

    let out = `current: ${peers.length} nodes and ${goodAvailabilityIndex + 1} nodes with availability 1.0 and ${
      peers.length - worstAvailabilityIndex
    } nodes with availability 0.0:\n`

    for (const peer of peers) {
      const entryIndex = this.findIndex(peer)

      if (entryIndex < 0) {
        continue
      }

      const entry = this.peers[entryIndex]

      const success =
        entry.heartbeatsSent > 0 ? ((entry.heartbeatsSuccess / entry.heartbeatsSent) * 100).toFixed() + '%' : '<new>'
      out += `- id: ${entry.id.toB58String()}, quality: ${this.qualityOf(entry.id).toFixed(
        2
      )} (backoff ${entry.backoff.toFixed()}, ${success} of ${entry.heartbeatsSent}) \n`
    }

    return out
  }
}

export default NetworkPeers
