import { type HeartbeatPingResult } from './heartbeat.js'
import type { PeerId } from '@libp2p/interface-peer-id'
import { randomSubset } from '@hoprnet/hopr-utils'

// const DEBUG_PREFIX = 'hopr-core:network-peers'
// const log = debug(DEBUG_PREFIX)
// const verbose = debug(DEBUG_PREFIX.concat(`:verbose`))

export type Entry = {
  id: PeerId
  heartbeatsSent: number
  heartbeatsSuccess: number
  lastSeen: number
  backoff: number // between 2 and MAX_BACKOFF
  quality: number
  origin: NetworkPeersOrigin
  ignoredAt?: number
}

export enum NetworkPeersOrigin {
  INITIALIZATION,
  NETWORK_REGISTRY,
  INCOMING_CONNECTION,
  OUTGOING_CONNECTION,
  STRATEGY_EXISTING_CHANNEL,
  STRATEGY_CONSIDERING_CHANNEL,
  STRATEGY_NEW_CHANNEL,
  MANUAL_PING,
  TESTING
}

const MIN_DELAY = 1000 // 1 sec (because this is multiplied by backoff, it will be half the actual minimum value.
const MAX_DELAY = 5 * 60 * 1000 // 5mins
const BACKOFF_EXPONENT = 1.5
export const MAX_BACKOFF = MAX_DELAY / MIN_DELAY
const BAD_QUALITY = 0.2 // Default quality for nodes we don't know about or which are considered offline.
const IGNORE_TIMEFRAME = 10 * 60 * 1000 // 10mins

function compareQualities(a: Entry, b: Entry, qualityOf: (id: PeerId) => number): number {
  const result = qualityOf(b.id) - qualityOf(a.id)

  if (result == 0) {
    return a.id.toString().localeCompare(b.id.toString(), 'en')
  } else {
    return result
  }
}

function printPeerOrigin(origin: NetworkPeersOrigin): string {
  switch (origin) {
    case NetworkPeersOrigin.INITIALIZATION:
      return 'node initialization'
    case NetworkPeersOrigin.NETWORK_REGISTRY:
      return 'registered in network registry'
    case NetworkPeersOrigin.INCOMING_CONNECTION:
      return 'incoming connection'
    case NetworkPeersOrigin.OUTGOING_CONNECTION:
      return `outgoing connection attempt`
    case NetworkPeersOrigin.STRATEGY_EXISTING_CHANNEL:
      return `strategy monitors existing channel`
    case NetworkPeersOrigin.STRATEGY_CONSIDERING_CHANNEL:
      return `strategy considers opening a channel`
    case NetworkPeersOrigin.STRATEGY_NEW_CHANNEL:
      return `strategy decided to open new channel`
    case NetworkPeersOrigin.MANUAL_PING:
      return `manuel ping`
    case NetworkPeersOrigin.TESTING:
      return `testing`
  }
}

function printEntries(
  entries: Map<string, Entry>,
  qualityOf: (id: PeerId) => number,
  networkQualityThreshold: number,
  prefix: string
): string {
  let bestAvailabilityNodes = 0
  let badAvailabilityNodes = 0
  let out = `${prefix}\n`
  let length = 0

  const peerIds: string[] = []
  for (const entry of entries.values()) {
    peerIds.push(entry.id.toString())
  }

  peerIds.sort((a: string, b: string) => compareQualities(entries.get(a), entries.get(b), qualityOf))

  for (const peer of peerIds) {
    const entry = entries.get(peer)

    const quality = qualityOf(entry.id)
    if (quality.toFixed(1) === '1.0') {
      bestAvailabilityNodes++
    } else if (quality < networkQualityThreshold) {
      badAvailabilityNodes++
    }

    if (out.length > prefix.length + 1) {
      out += '\n'
    }

    const success =
      entry.heartbeatsSent > 0 ? ((entry.heartbeatsSuccess / entry.heartbeatsSent) * 100).toFixed() + '%' : '<new>'
    out += `- id: ${entry.id.toString()}, `
    out += `quality: ${qualityOf(entry.id).toFixed(2)}, `
    out += `backoff: ${entry.backoff.toFixed()} (${success} of ${entry.heartbeatsSent}), `
    out += `origin: ${printPeerOrigin(entry.origin)}`

    length++
  }

  if (out.length == prefix.length + 1) {
    return 'no connected peers'
  } else {
    out += '\n'
  }

  const msgTotalNodes = `${length} node${length == 1 ? '' : 's'} in total`
  const msgBestNodes = `${bestAvailabilityNodes} node${bestAvailabilityNodes == 1 ? '' : 's'} with quality 1.0`
  const msgBadNodes = `${badAvailabilityNodes} node${
    badAvailabilityNodes == 1 ? '' : 's'
  } with quality below ${networkQualityThreshold}`
  out += `network peers status: ${msgTotalNodes}, ${msgBestNodes}, ${msgBadNodes}\n`

  return out
}

function nextPing(e: Entry): number {
  // Exponential backoff
  const delay = Math.min(MAX_DELAY, Math.pow(e.backoff, BACKOFF_EXPONENT) * MIN_DELAY)
  return e.lastSeen + delay
}

/**
 *
 */
class NetworkPeers {
  // peerId.toString() -> latest measurement
  private entries: Map<string, Entry>
  // peerId.toString() -> ignore since timestamp
  private ignoredEntries: Map<string, number>

  // static set of excluded peers
  private excludedPeers: Set<string>

  constructor(
    existingPeers: PeerId[],
    excludedPeers: PeerId[] = [], // populated only by constructor, does not change during runtime
    private networkQualityThreshold: number,
    private onPeerOffline?: (peer: PeerId) => void
  ) {
    if (this.networkQualityThreshold < BAD_QUALITY) {
      throw Error(`Invalid configuration networkQuality must be greater or equal to BAD_QUALITY`)
    }

    this.entries = new Map<string, Entry>()
    this.ignoredEntries = new Map<string, number>()

    this.excludedPeers = new Set<string>(excludedPeers.map((p: PeerId) => p.toString()))

    // register all existing peers
    for (const peer of existingPeers) {
      this.register(peer, NetworkPeersOrigin.INITIALIZATION)
    }
  }

  /**
   * Returns the quality of the node, where
   * 0.0 => completely unreliable / offline or unknown
   * 1.0 => completely reliable / online
   * @param peerId id for which to get quality
   * @returns a float between 0.0 and 1.0
   */
  public qualityOf(peerId: PeerId): number {
    const entry = this.entries.get(peerId.toString())

    if (!entry) {
      // Lower than anything else
      return 0.0
    }

    if (entry.heartbeatsSent > 0) {
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
    const id = peerId.toString()
    const entry = this.entries.get(id)
    if (entry) return entry
    throw Error(`Entry for ${id} does not exist`)
  }

  public pingSince(thresholdTime: number): PeerId[] {
    const toPing: PeerId[] = []

    // Returns list of filtered nodes, statically ordered after *insertion* into Map
    for (const entry of this.entries.values()) {
      if (nextPing(entry) < thresholdTime) {
        toPing.push(entry.id)
      }
    }

    // Ping most recently seen nodes last
    return toPing.sort(
      (a: PeerId, b: PeerId) => this.entries.get(a.toString()).lastSeen - this.entries.get(b.toString()).lastSeen
    )
  }

  public updateRecord(pingResult: HeartbeatPingResult): void {
    const id = pingResult.destination
    const previousEntry = this.entries.get(id.toString())
    if (!previousEntry) return

    if (pingResult.lastSeen < 0) {
      this.onFailedPing(id)
    } else {
      this.onSuccessfulPing(id)
    }
  }

  private onFailedPing(id: PeerId) {
    const previousEntry = this.entries.get(id.toString())

    const newEntry = {
      id,
      heartbeatsSent: previousEntry.heartbeatsSent + 1,
      lastSeen: Date.now(),
      heartbeatsSuccess: previousEntry.heartbeatsSuccess,
      backoff: Math.min(MAX_BACKOFF, Math.pow(previousEntry.backoff, BACKOFF_EXPONENT)),
      quality: Math.max(0, previousEntry.quality - 0.1),
      origin: previousEntry.origin
    }

    if (newEntry.quality < this.networkQualityThreshold) {
      // trigger callback first to cut connections
      this.onPeerOffline?.(id)
    }

    // check if this node is considered offline and should be removed
    if (newEntry.quality < BAD_QUALITY) {
      // delete peer from internal store
      this.entries.delete(id.toString())
      // add entry to temporarily ignored peers
      // Create or overwrite entry in ignore list
      this.ignoredEntries.set(newEntry.id.toString(), Date.now())
      // done, return early so the rest can update the entry instead
      return
    }

    this.entries.set(id.toString(), newEntry)
  }

  private onSuccessfulPing(id: PeerId) {
    const previousEntry = this.entries.get(id.toString())

    this.entries.set(id.toString(), {
      id,
      heartbeatsSent: previousEntry.heartbeatsSent + 1,
      lastSeen: Date.now(),
      heartbeatsSuccess: previousEntry.heartbeatsSuccess + 1,
      backoff: 2, // RESET - to back down: Math.pow(entry.backoff, 1/BACKOFF_EXPONENT)
      quality: Math.min(1, previousEntry.quality + 0.1),
      origin: previousEntry.origin
    })
  }

  /**
   * Creates a random sample of stored peers
   *
   * @param size desired number of peers
   * @param filter allow only selected peerIds
   * @returns a randomly picked array of peers
   */
  public randomSubset(size: number, filter?: (peer: PeerId) => boolean): PeerId[] {
    const peers = Array.from(this.entries.values())
    return randomSubset(
      peers,
      Math.min(size, peers.length),
      filter != null ? (entry: Entry) => filter(entry.id) : undefined
    ).map((e: Entry) => e.id)
  }

  public register(peerId: PeerId, origin: NetworkPeersOrigin) {
    const id = peerId.toString()
    const now = Date.now()

    // Assumes that all maps / sets are disjoint
    const hasEntry = this.entries.has(id)
    const isExcluded = !hasEntry && this.excludedPeers.has(id)
    let isIgnored = !isExcluded && this.ignoredEntries.has(id)

    // the peer is excluded or denied
    if (isExcluded) {
      // not adding peer
      return
    }

    if (isIgnored) {
      const ignoreTimestamp: undefined | number = this.ignoredEntries.get(peerId.toString())

      // Must test for undefined because if(1) is treated as if(true)
      if (ignoreTimestamp != undefined && ignoreTimestamp + IGNORE_TIMEFRAME < now) {
        // release and continue
        this.ignoredEntries.delete(peerId.toString())
        isIgnored = false
      }
    }

    // does not have peer and it's not excluded or denied
    if (!hasEntry && !isIgnored) {
      this.entries.set(id, {
        id: peerId,
        heartbeatsSent: 0,
        heartbeatsSuccess: 0,
        lastSeen: now,
        backoff: 2,
        quality: BAD_QUALITY,
        origin
      })
    }
  }

  public unregister(peerId: PeerId) {
    this.entries.delete(peerId.toString())
  }

  public has(peerId: PeerId): boolean {
    return this.entries.has(peerId.toString())
  }

  public length(): number {
    return this.entries.size
  }

  public all(): PeerId[] {
    return Array.from(this.getAllEntries()).map((entry) => entry.id)
  }

  /**
   * @returns a string describing the connection quality of all connected peers
   */
  public debugLog(prefix: string = ''): string {
    return printEntries(this.entries, this.qualityOf.bind(this), this.networkQualityThreshold, prefix)
  }

  public getAllEntries(): IterableIterator<Entry> {
    return this.entries.values()
  }

  public getAllIgnored(): IterableIterator<string> {
    return this.ignoredEntries.keys()
  }
}

export default NetworkPeers
