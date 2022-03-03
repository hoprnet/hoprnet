import { type HeartbeatPingResult } from './heartbeat'
import PeerId from 'peer-id'
import { randomSubset } from '@hoprnet/hopr-utils'
import { NETWORK_QUALITY_THRESHOLD } from '../constants'

export type Entry = {
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
  private peers: Map<string, Entry> = new Map()

  constructor(
    existingPeers: Array<PeerId>,
    private exclude: PeerId[] = [],
    private onPeerOffline?: (peer: PeerId) => void
  ) {
    // register all existing peers
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
  public qualityOf(peerId: PeerId): number {
    const entry = this.peers.get(peerId.toB58String())
    if (entry && entry.heartbeatsSent > 0) {
      /*
      return entry.heartbeatsSuccess / entry.heartbeatsSent
      */
      return entry.lastTen
    }
    return UNKNOWN_Q
  }

  /**
   * @param peerId of the node we want to get the connection info for
   * @returns various information about the connection, throws error if peerId doesn't exist
   */
  public getConnectionInfo(peerId: PeerId): Entry {
    const id = peerId.toB58String()
    const entry = this.peers.get(id)
    if (entry) return entry
    throw Error(`Entry for ${id} does not exist`)
  }

  public pingSince(thresholdTime: number): PeerId[] {
    const toPing: PeerId[] = []
    for (const entry of this.peers.values()) {
      if (this.nextPing(entry) < thresholdTime) {
        toPing.push(entry.id)
      }
    }

    return toPing
  }

  public updateRecord(pingResult: HeartbeatPingResult): void {
    const id = pingResult.destination.toB58String()
    const previousEntry = this.peers.get(id)
    if (!previousEntry) return

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

    this.peers.set(id, newEntry)
  }

  // Get a random sample peers.
  public randomSubset(size: number, filter?: (peer: PeerId) => boolean): PeerId[] {
    const peers = Array.from(this.peers.values())
    return randomSubset(
      peers,
      Math.min(size, peers.length),
      filter != null ? (entry: Entry) => filter(entry.id) : undefined
    ).map((e: Entry) => e.id)
  }

  public register(peerId: PeerId) {
    const id = peerId.toB58String()

    // does not have peer and it's not excluded
    if (!this.peers.has(id) && this.exclude.findIndex((p: PeerId) => peerId.equals(p)) < 0) {
      this.peers.set(id, {
        id: peerId,
        heartbeatsSent: 0,
        heartbeatsSuccess: 0,
        lastSeen: Date.now(),
        backoff: 2,
        lastTen: UNKNOWN_Q
      })
    }
  }

  public has(peerId: PeerId): boolean {
    return this.peers.has(peerId.toB58String())
  }

  public length(): number {
    return this.peers.size
  }

  public all(): PeerId[] {
    return Array.from(this.peers.values()).map((peer) => peer.id)
  }

  /**
   * @deprecated Used by API v1
   * @returns a string describing the connection quality of all connected peers
   */
  public debugLog(): string {
    const peers = Array.from(this.peers.values())
    if (peers.length == 0) {
      return 'no connected peers'
    }
    let out = ''
    out += `current nodes:\n`
    peers
      .sort((a, b) => {
        return this.qualityOf(b.id) - this.qualityOf(a.id)
      })
      .forEach((e: Entry) => {
        const success =
          e.heartbeatsSent > 0 ? ((e.heartbeatsSuccess / e.heartbeatsSent) * 100).toFixed() + '%' : '<new>'
        out += `- id: ${e.id.toB58String()}, quality: ${this.qualityOf(e.id).toFixed(
          2
        )} (backoff ${e.backoff.toFixed()}, ${success} of ${e.heartbeatsSent}) \n`
      })
    return out
  }
}

export default NetworkPeers
