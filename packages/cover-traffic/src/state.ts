import BN from 'bn.js'
import { PublicKey, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { findChannel, importance } from './utils'
import fs from 'fs'

export type ChannelData = {
  channel: ChannelEntry
  sendAttempts: number
  forwardAttempts: number
}

export type OpenChannels = {
  destination: PublicKey
  latestQualityOf: number
  openFrom: number // timestamp (in milliseconds) when the CT channel is opened
}

export type State = {
  nodes: Record<string, PeerData>
  channels: Record<string, ChannelData>
  log: string[]
  ctChannels: OpenChannels[]
  block: BN
  messageFails: Record<string, number>
}

export type PeerData = {
  id: any //PeerId,
  pub: PublicKey
  multiaddrs: any
}

const DB = './ct.json'

export class PersistedState {
  // Quick and dirty DB.
  // Caveats:
  // - Must live in same timeline as the hoprdb, as it relies on
  //   the indexer being in the same state.
  private _data: State

  constructor(private update: (s: State) => void) {
    if (fs.existsSync(DB)) {
      this.load()
    } else {
      this._data = {
        nodes: {},
        channels: {},
        log: [],
        ctChannels: [],
        messageFails: {},
        block: new BN('0')
      }
    }
  }

  load(): void {
    const json = JSON.parse(fs.readFileSync(DB, 'utf8'))
    this._data = {
      nodes: {},
      channels: {},
      log: ['loaded data'],
      ctChannels: json.ctChannels.map((p) => ({
        destination: PublicKey.fromPeerId(PeerId.createFromB58String(p.destination)),
        latestQualityOf: 0,
        openFrom: p.openFrom
      })),
      messageFails: {},
      block: new BN(json.block)
    }
    json.nodes.forEach((n) => {
      const id = PeerId.createFromB58String(n.id)
      this._data.nodes[id.toB58String()] = { id, pub: PublicKey.fromPeerId(id), multiaddrs: [] }
    })

    json.channels.forEach((c) => {
      const channel = ChannelEntry.deserialize(Uint8Array.from(Buffer.from(c.channel, 'base64')))
      this._data.channels[channel.getId().toHex()] = {
        channel,
        forwardAttempts: c.forwardAttempts,
        sendAttempts: c.sendAttempts
      }
    })
  }

  get(): State {
    return this._data
  }

  set(s: State): void {
    this._data = s
    fs.writeFileSync(
      DB,
      JSON.stringify({
        nodes: Object.values(s.nodes).map((n: PeerData) => ({
          id: n.id.toB58String(),
          multiaddrs: n.multiaddrs.map((m) => m.toString())
        })),
        channels: Object.values(s.channels).map((c) => ({
          channel: Buffer.from(c.channel.serialize()).toString('base64'),
          forwardAttempts: c.forwardAttempts,
          sendAttempts: c.sendAttempts
        })),
        ctChannels: s.ctChannels.map((o: OpenChannels) => ({
          destination: o.destination.toB58String(),
          openFrom: o.openFrom
        })),
        block: s.block.toString()
      }),
      'utf8'
    )
    this.update(s)
    return
  }

  setChannel(channel: ChannelEntry): void {
    const state = this.get()
    if (state.channels[channel.getId().toHex()]) {
      state.channels[channel.getId().toHex()].channel = channel
    } else {
      state.channels[channel.getId().toHex()] = {
        channel,
        sendAttempts: 0,
        forwardAttempts: 0
      }
    }
    this.set(state)
  }

  setNode(peer: PeerData): void {
    const state = this.get()
    state.nodes[peer.id.toB58String()] = {
      id: peer.id,
      multiaddrs: peer.multiaddrs,
      pub: PublicKey.fromPeerId(peer.id)
    }
    this.set(state)
  }

  setCTChannels(channels: OpenChannels[]): void {
    const state = this.get()
    state.ctChannels = channels
    this.set(state)
  }

  findChannelsFrom(p: PublicKey): ChannelEntry[] {
    return Object.values(this.get().channels)
      .filter((c: ChannelData) => c.channel.source.eq(p))
      .map((c) => c.channel)
  }

  log(...args: String[]): void {
    const s = this.get()
    s.log.push(args.join(' '))
    this.set(s)
  }

  setBlock(block: BN): void {
    const s = this.get()
    s.block = block
    this.set(s)
  }

  getNode(b58String: string): PeerData {
    const s = this.get()
    return s.nodes[b58String]
  }

  findChannel(src: PublicKey, dest: PublicKey): ChannelEntry {
    const s = this.get()
    return findChannel(src, dest, s)
  }

  weightedRandomChoice(): PublicKey {
    const s = this.get()
    if (Object.keys(s.nodes).length == 0) {
      throw new Error('no nodes to pick from')
    }
    const weights: Record<string, BN> = {}
    let total = new BN('0')
    const ind = Math.random()

    for (const p of Object.values(s.nodes)) {
      weights[p.pub.toHex()] = importance(p.pub, s)
      total = total.add(weights[p.pub.toHex()])
    }

    if (total.lten(0)) {
      // No important nodes - let's pick a random node for now.
      const index = Math.floor(ind * Object.keys(s.nodes).length)
      return Object.values(s.nodes)[index].pub
    }

    let interval = total.muln(ind)
    for (let node of Object.keys(weights)) {
      interval = interval.sub(weights[node])
      if (interval.lte(new BN('0'))) {
        return PublicKey.fromString(node)
      }
    }
    throw new Error('wtf')
  }

  async incrementSent(_p: PublicKey) {
    // const s = await this.get()
    // TODO init
    //s.channels[p.toB58String()].sendAttempts ++
  }

  async incrementForwards(_p: PublicKey) {
    //const s = await this.get()
    // TODO init
    //s.ctSent[p.toB58String()].forwardAttempts++
  }

  openChannelCount(): number {
    const s = this.get()
    return Object.values(s.channels).filter((x) => x.channel.status != ChannelStatus.Closed).length
  }

  messageFails(dest: PublicKey): number {
    return this.get().messageFails[dest.toB58String()] || 0
  }

  incrementMessageFails(dest: PublicKey): void {
    const s = this.get()
    const prev = s.messageFails[dest.toB58String()] || 0
    s.messageFails[dest.toB58String()] = prev + 1
    this.set(s)
  }

  resetMessageFails(dest: PublicKey): void {
    const s = this.get()
    s.messageFails[dest.toB58String()] = 0
    this.set(s)
  }
}
