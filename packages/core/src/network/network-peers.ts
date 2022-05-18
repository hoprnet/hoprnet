import { type HeartbeatPingResult } from './heartbeat'
import PeerId from 'peer-id'
import { randomSubset, debug } from '@hoprnet/hopr-utils'
import { NETWORK_QUALITY_THRESHOLD } from '../constants'

const log = debug('hopr-core:network-peers')

export type Entry = {
  id: PeerId
  isPublic: boolean // Indicates whether the node is known to be public
  heartbeatsSent: number
  heartbeatsSuccess: number
  lastSeen: number
  backoff: number // between 2 and MAX_BACKOFF
  quality: number
  origin: string
  ignoredAt?: number
}

const MIN_DELAY = 1000 // 1 sec (because this is multiplied by backoff, it will be half the actual minimum value.
const MAX_DELAY = 5 * 60 * 1000 // 5mins
const BACKOFF_EXPONENT = 1.5
export const MAX_BACKOFF = MAX_DELAY / MIN_DELAY
const BAD_QUALITY = 0.2 // Default quality for nodes we don't know about or which are considered offline.
const IGNORE_TIMEFRAME = 10 * 60 * 1000 // 10mins

function nextPing(e: Entry): number {
  // Exponential backoff
  const delay = Math.min(MAX_DELAY, Math.pow(e.backoff, BACKOFF_EXPONENT) * MIN_DELAY)
  return e.lastSeen + delay
}

class NetworkPeers {
  private entries: Map<string, Entry> = new Map()
  private ignoredEntries: Entry[] = []
  // peers which were denied connection via the HoprNetworkRegistry
  private deniedEntries: Map<string, Pick<Entry, 'id' | 'origin'>> = new Map()

  constructor(
    existingPeers: PeerId[],
    private excludedPeers: PeerId[] = [], // populated only by constructor, does not change during runtime
    private onPeerOffline?: (peer: PeerId) => void
  ) {
    // register all existing peers
    for (const peer of existingPeers) {
      this.register(peer, 'network peers initialization')
    }
  }

  // @returns a float between 0 (completely unreliable) and 1 (completely
  // reliable) estimating the quality of service of a peer's network connection
  public qualityOf(peerId: PeerId): number {
    const entry = this.entries.get(peerId.toB58String())
    if (entry && entry.heartbeatsSent > 0) {
      /*
      return entry.heartbeatsSuccess / entry.heartbeatsSent
      */
      return entry.quality
    }
    return BAD_QUALITY
  }

  /**
   * @param peerId of the node we want to get the connection info for
   * @returns various information about the connection, throws error if peerId doesn't exist
   */
  public getConnectionInfo(peerId: PeerId): Entry {
    const id = peerId.toB58String()
    const entry = this.entries.get(id)
    if (entry) return entry
    throw Error(`Entry for ${id} does not exist`)
  }

  public pingSince(thresholdTime: number): PeerId[] {
    const toPing: PeerId[] = []
    for (const entry of this.entries.values()) {
      if (nextPing(entry) < thresholdTime) {
        toPing.push(entry.id)
      }
    }

    return toPing
  }

  public updateRecord(pingResult: HeartbeatPingResult): void {
    const id = pingResult.destination.toB58String()
    const previousEntry = this.entries.get(id)
    if (!previousEntry) return

    let newEntry: Entry

    if (pingResult.lastSeen < 0) {
      // failed ping
      newEntry = {
        id: pingResult.destination,
        isPublic: previousEntry.isPublic,
        heartbeatsSent: previousEntry.heartbeatsSent + 1,
        lastSeen: Date.now(),
        heartbeatsSuccess: previousEntry.heartbeatsSuccess,
        backoff: Math.min(MAX_BACKOFF, Math.pow(previousEntry.backoff, BACKOFF_EXPONENT)),
        quality: Math.max(0, previousEntry.quality - 0.1),
        origin: previousEntry.origin
      }
      if (newEntry.quality < NETWORK_QUALITY_THRESHOLD) {
        // trigger callback first to cut connections
        this.onPeerOffline?.(pingResult.destination)

        // check if this node is considered offline and should be removed
        if (newEntry.quality < BAD_QUALITY) {
          // delete peer from internal store
          this.entries.delete(id)
          // add entry to temporarily ignored peers
          this.ignoreEntry(newEntry)
          // done, return early so the rest can update the entry instead
          return
        }
      }
    } else {
      // successful ping
      newEntry = {
        id: pingResult.destination,
        isPublic: previousEntry.isPublic,
        heartbeatsSent: previousEntry.heartbeatsSent + 1,
        lastSeen: Date.now(),
        heartbeatsSuccess: previousEntry.heartbeatsSuccess + 1,
        backoff: 2, // RESET - to back down: Math.pow(entry.backoff, 1/BACKOFF_EXPONENT)
        quality: Math.min(1, previousEntry.quality + 0.1),
        origin: previousEntry.origin
      }
    }

    // update peer entry if still considered ok to keep
    this.entries.set(id, newEntry)
  }

  // Get a random sample peers.
  public randomSubset(size: number, filter?: (peer: PeerId) => boolean): PeerId[] {
    const peers = Array.from(this.entries.values())
    return randomSubset(
      peers,
      Math.min(size, peers.length),
      filter != null ? (entry: Entry) => filter(entry.id) : undefined
    ).map((e: Entry) => e.id)
  }

  public register(peerId: PeerId, origin: string) {
    const id = peerId.toB58String()
    const now = Date.now()
    const hasEntry = this.entries.has(id)
    const isExcluded = !hasEntry && this.excludedPeers.some((p) => p.equals(peerId))
    const isDenied = !hasEntry && this.deniedEntries.has(id)

    log('registering peer', id, { hasEntry, isExcluded, isDenied })

    // does not have peer and it's not excluded or denied
    if (!hasEntry && !isExcluded && !isDenied) {
      this.entries.set(id, {
        id: peerId,
        isPublic: false,
        heartbeatsSent: 0,
        heartbeatsSuccess: 0,
        lastSeen: now,
        backoff: 2,
        quality: BAD_QUALITY,
        origin
      })
    }

    // the peer is excluded or denied
    if (isExcluded || isDenied) {
      return
    }

    const ignoredIndex = this.ignoredEntries.findIndex((e: Entry) => e.id.equals(peerId) && e.origin == origin)
    if (ignoredIndex >= 0) {
      // the peer is temporarily ignored, release if time has passed
      const ignoredEntry = this.ignoredEntries[ignoredIndex]
      if (ignoredEntry.ignoredAt + IGNORE_TIMEFRAME < now) {
        // release and continue
        this.unignoreEntry(ignoredEntry)
      } else {
        // ignore still valid, thus skipping this registration
        return
      }
    }
  }

  public has(peerId: PeerId): boolean {
    return this.entries.has(peerId.toB58String())
  }

  public length(): number {
    return this.entries.size
  }

  public allEntries(): Entry[] {
    return Array.from(this.entries.values())
  }

  public all(): PeerId[] {
    return this.allEntries().map((entry) => entry.id)
  }

  /**
   * @returns a string describing the connection quality of all connected peers
   */
  public debugLog(): string {
    if (this.entries.size === 0) return 'no connected peers'

    const peers = this.all()

    // Sort a copy of peers in-place
    peers.sort((a, b) => this.qualityOf(b) - this.qualityOf(a))

    let bestAvailabilityNodes = 0
    let badAvailabilityNodes = 0
    let out = ''

    for (const peer of peers) {
      if (!this.has(peer)) {
        continue
      }

      const entry = this.entries.get(peer.toB58String())

      const quality = this.qualityOf(peer)
      if (quality.toFixed(1) === '1.0') {
        bestAvailabilityNodes++
      } else if (quality < NETWORK_QUALITY_THRESHOLD) {
        badAvailabilityNodes++
      }

      const success =
        entry.heartbeatsSent > 0 ? ((entry.heartbeatsSuccess / entry.heartbeatsSent) * 100).toFixed() + '%' : '<new>'
      out += `- id: ${entry.id.toB58String()}, `
      out += `quality: ${this.qualityOf(entry.id).toFixed(2)}, `
      out += `backoff: ${entry.backoff.toFixed()} (${success} of ${entry.heartbeatsSent}), `
      out += `origin: ${entry.origin}`
      out += '\n'
    }

    const msgTotalNodes = `${peers.length} node${peers.length == 1 ? '' : 's'} in total`
    const msgBestNodes = `${bestAvailabilityNodes} node${bestAvailabilityNodes == 1 ? '' : 's'} with quality 1.0`
    const msgBadNodes = `${badAvailabilityNodes} node${badAvailabilityNodes == 1 ? '' : 's'} with quality below 0.5`
    out += `network peers status: ${msgTotalNodes}, ${msgBestNodes}, ${msgBadNodes}\n`

    return out
  }

  private ignoreEntry(entry: Entry): void {
    const index = this.ignoredEntries.findIndex((e: Entry) => e.id.equals(entry.id) && e.origin == entry.origin)

    if (index < 0) {
      entry.ignoredAt = Date.now()
      this.ignoredEntries.push(entry)
    }
  }

  private unignoreEntry(entry: Entry): void {
    const index = this.ignoredEntries.findIndex((e: Entry) => e.id.equals(entry.id) && e.origin == entry.origin)

    if (index >= 0) {
      this.ignoredEntries.splice(index, 1)
    }
  }

  public getAllDenied(): Pick<Entry, 'id' | 'origin'>[] {
    return Array.from(this.deniedEntries.values())
  }

  public addPeerToDenied(peerId: PeerId, origin: string): void {
    const peerIdStr = peerId.toB58String()
    log('adding peer to denied', peerIdStr)
    this.deniedEntries.set(peerIdStr, { id: peerId, origin })
  }

  public removePeerFromDenied(peerId: PeerId): void {
    const peerIdStr = peerId.toB58String()
    log('removing peer from denied', peerIdStr)
    this.deniedEntries.delete(peerIdStr)
  }
}

export default NetworkPeers
