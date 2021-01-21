import { randomSubset } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'

type Entry = {
  id: PeerId
  heartbeatsSent: number
  heartbeatsSuccess: number
  lastSeen: number
  backoff: number
  //lastTen: number
}

const MIN_DELAY = 3000 // 3 secs
const MAX_DELAY = 5 * 60 * 1000 // 5mins
const BACKOFF_EXPONENT = 1.5

class NetworkPeers {
  private peers: Entry[]

  private find(peer: PeerId): Entry | undefined {
    return this.peers.find((x) => x.id.toB58String() === peer.toB58String())
  }

  constructor(existingPeers: Array<PeerId>) {
    this.peers = []

    for (const peer of existingPeers) {
      this.register(peer)
    }
  }

  private nextPing(e: Entry): number{
    // Exponential backoff
    const delay = Math.min(MAX_DELAY, Math.pow(e.backoff, BACKOFF_EXPONENT) * MIN_DELAY)
    return e.lastSeen + delay 
  }

  // @returns a float between 0 (completely unreliable) and 1 (completely
  // reliable) estimating the quality of service of a peer's network connection
  public qualityOf(peer: PeerId): number {
    let entry = this.find(peer)
    if (entry && entry.heartbeatsSent > 0) {
      return entry.heartbeatsSuccess / entry.heartbeatsSent
    }
    // @TODO
    return 0.2 // Unknown // TBD
  }

  public pingSince(thresholdTime: number): PeerId[]{
    return this.peers
               .filter(entry => this.nextPing(entry) > thresholdTime)
               .map(x => x.id) 
  }


  public async ping(peer: PeerId, interaction: (peerID: PeerId) => Promise<boolean>): Promise<void> {

    const entry = this.find(peer)
    if (!entry) throw new Error('Cannot ping ' + peer.toB58String());

    entry.heartbeatsSent++
    entry.lastSeen = Date.now()
    const result = await interaction(entry.id)
    if (result) {
      entry.heartbeatsSuccess++
      entry.backoff = 1 // RESET - to back down: Math.pow(entry.backoff, 1/BACKOFF_EXPONENT)
    } else {

      entry.backoff = Math.pow(entry.backoff, BACKOFF_EXPONENT)
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
        this.peers.push({ id,
          heartbeatsSent: 0,
          heartbeatsSuccess: 0,
          lastSeen: Date.now(),
          backoff: 1,
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
    let out = ''
    out += `current nodes:\n`
    this.peers.forEach((e: Entry) => (out += `- id: ${e.id.toB58String()}, quality: ${this.qualityOf(e.id)}\n`))
    return out
  }
}

export default NetworkPeers
