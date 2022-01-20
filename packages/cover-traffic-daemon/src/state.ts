import BN from 'bn.js'
import { PublicKey, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { findChannel, importance } from './utils'
import fs from 'fs'

export type ChannelData = {
  channel: ChannelEntry
  // number of attempts of a channel to receive and send CT packets as the 1st hop (aka the recipient of CT channel)
  sendAttempts: number
  // number of attempts of a channel to forward packets (aka as intermediate hops other than the 1st hop).
  forwardAttempts: number
}

export type OpenChannels = {
  destination: PublicKey
  latestQualityOf: number
  openFrom: number // timestamp (in milliseconds) when the CT channel is opened
}

export type State = {
  nodes: Record<string, PeerData>
  // channels indexed by its channelId
  channels: Record<string, ChannelData>
  // currently non-closed cover traffic channels
  ctChannels: OpenChannels[]
  block: BN
  // number of failed messages indexed by the base58-encoded string of node id.
  messageFails: Record<string, number>
  // number of messages being successfully sent out by the CT node
  messageTotalSuccess: number
}

export type PeerData = {
  id: any //PeerId type, as implemented in IPFS
  pub: PublicKey
  multiaddrs: any[] // Multiaddress type
}

export class PersistedState {
  // Quick and dirty DB.
  // Caveats:
  // - Must live in same timeline as the hoprdb, as it relies on
  //   the indexer being in the same state.
  private _data: State

  /**
   * Initiate the persisted state of the network attached to the CT node
   * @param update function that is called at every change of the state
   */
  constructor(private update: (s: State) => void, private db_path: string) {
    if (fs.existsSync(this.db_path)) {
      this.load()
    } else {
      this._data = {
        nodes: {},
        channels: {},
        ctChannels: [],
        messageFails: {},
        messageTotalSuccess: 0,
        block: new BN('0')
      }
    }
  }

  /**
   * Load the exisitng cover traffic state, where the path is defined in `DB`
   */
  load(): void {
    const json = JSON.parse(fs.readFileSync(this.db_path, 'utf8'))
    this._data = {
      nodes: {},
      channels: {},
      ctChannels: json.ctChannels.map((p) => ({
        destination: PublicKey.fromPeerId(PeerId.createFromB58String(p.destination)),
        latestQualityOf: 0,
        openFrom: p.openFrom
      })),
      messageFails: {},
      messageTotalSuccess: 0,
      block: new BN(json.block)
    }

    // node ids are encoded in base58 strings
    json.nodes.forEach((n) => {
      const id = PeerId.createFromB58String(n.id)
      this._data.nodes[id.toB58String()] = { id, pub: PublicKey.fromPeerId(id), multiaddrs: [] }
    })

    // channel entries are encoded in base64 strings
    json.channels.forEach((c) => {
      const channel = ChannelEntry.deserialize(Uint8Array.from(Buffer.from(c.channel, 'base64')))
      this._data.channels[channel.getId().toHex()] = {
        channel,
        forwardAttempts: c.forwardAttempts,
        sendAttempts: c.sendAttempts
      }
    })
  }

  /**
   * Retrieve the persisted state
   * @returns State network state
   */
  get(): State {
    return this._data
  }

  /**
   * Write a network state into the persisted state as a string, and execute the
   * update at the end of the writing.
   * @param s persisted network state
   */
  set(s: State): void {
    this._data = s
    fs.writeFileSync(
      this.db_path,
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

  /**
   * Update a channel. If the channel does not exist in the state,
   * it inititalizes it with a clean history of attempts.
   * @param channel ChannelEntry new channel state to be updated in the persisted state.
   */
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

  /**
   * When the indexer discovers a new node joining the network, add it to the persisted state.
   * @param peer {id: PeerId, multiaddrs: MultiAddrs[]} Object emitted by the indexer on 'peer'
   */
  setNode(peer: PeerData): void {
    const state = this.get()
    state.nodes[peer.id.toB58String()] = {
      id: peer.id,
      multiaddrs: peer.multiaddrs,
      pub: PublicKey.fromPeerId(peer.id)
    }
    this.set(state)
  }

  /**
   * Update the list of all the non-closed cover traffic channels with the current
   * cover traffic node as `source`
   * @param channels Channels opened with the cover traffic node as `source`
   */
  setCTChannels(channels: OpenChannels[]): void {
    const state = this.get()
    state.ctChannels = channels
    this.set(state)
  }

  /**
   * Get channels opened from a node with a given public key
   * @param p Public key of the `source` node
   * @returns a list of channel entries where the `source` is the given public key
   */
  findChannelsFrom(p: PublicKey): ChannelEntry[] {
    return Object.values(this.get().channels)
      .filter((c: ChannelData) => c.channel.source.eq(p))
      .map((c) => c.channel)
  }

  /**
   * Update the lastest block number being picked up by the indexer.
   * It indicates the syncing stage of the persisted state.
   * @param block Latest block number (a big number) listened by the indexer.
   */
  setBlock(block: BN): void {
    const s = this.get()
    s.block = block
    this.set(s)
  }

  /**
   * Get the PeerData associated with a node from the persisted state
   * @param b58String Node ID encoded in base58
   * @returns PeerData of the given ID.
   */
  getNode(b58String: string): PeerData {
    const s = this.get()
    return s.nodes[b58String]
  }

  /**
   * From all the network channels saved in the persisted state, find the
   * ChannelEntry that is between the provided source and destination.
   * @param src Public key of the `source` of the channel
   * @param dest Public key of the `destination` of the channel
   * @returns ChannelEntry between `source` and `destination`, undefined otherwise.
   */
  findChannel(src: PublicKey, dest: PublicKey): ChannelEntry {
    const s = this.get()
    return findChannel(src, dest, s)
  }

  /**
   * Randomly return a node in the network. The chance of being picked up is
   * in proportion to its weights, which is calculated through the formula of
   * `importance`.
   * @returns a Public key of the node being randomed chosen based on its
   * weight, throw an error otherwise.
   */
  weightedRandomChoice(): PublicKey {
    const s = this.get()
    if (Object.keys(s.nodes).length == 0) {
      throw new Error('no nodes to pick from')
    }
    // key: Public key of a node, value: Importance score of a node
    const weights: Record<string, BN> = {}
    let total = new BN('0')
    const ind = Math.random()

    // for all the nodes in the network, set importance score as its weight and calculate the sum of all weights
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

  /**
   * Increase the number of attempts for a channel to send out packets (as the 1st hop, aka the destination of CT channel) by 1
   * @param source Public key of the message sender.
   * @param destination Public key of the message sender.
   */
  async incrementSent(source: PublicKey, destination: PublicKey) {
    const s = this.get()
    const channel = findChannel(source, destination, s)
    if (channel) {
      const channelId = channel.getId().toHex()
      const prev = s.channels[channelId].sendAttempts || 0
      s.channels[channelId].sendAttempts = prev + 1
      this.set(s)
    }
  }

  /**
   * Increase the number of attempts for a channel to forward packets by 1
   * @param source Public key of the message sender.
   * @param destination Public key of the message sender.
   */
  async incrementForwards(source: PublicKey, destination: PublicKey) {
    const s = this.get()
    const channel = findChannel(source, destination, s)
    if (channel) {
      const channelId = channel.getId().toHex()
      const prev = s.channels[channelId].forwardAttempts || 0
      s.channels[channelId].forwardAttempts = prev + 1
      this.set(s)
    }
  }

  /**
   * Returns the number of open channels in the network
   * @returns number of all the open channels
   */
  openChannelCount(): number {
    const s = this.get()
    return Object.values(s.channels).filter((x) => x.channel.status != ChannelStatus.Closed).length
  }

  /**
   * Get the number of total messages that are sent out from the current CT node
   * @returns number of total messages. If none, returns zero.
   */
  messageTotalSuccess(): number {
    return this.get().messageTotalSuccess || 0
  }

  /**
   * Update the total number of sent messages from the current CT node
   */
  incrementMessageTotalSuccess(): void {
    const s = this.get()
    const prev = s.messageTotalSuccess || 0
    s.messageTotalSuccess = prev + 1
    this.set(s)
  }

  /**
   * Get the number of failed messages that should have been sent to the destination
   * @param dest Public key of the destination that supposed to received messages but failed.
   * @returns number of failed messages. If none, returns zero.
   */
  messageFails(dest: PublicKey): number {
    return this.get().messageFails[dest.toB58String()] || 0
  }

  /**
   * Update the number of failed messages sending to a node
   * @param dest Public key of the destination that supposed to received messages but failed.
   */
  incrementMessageFails(dest: PublicKey): void {
    const s = this.get()
    const prev = s.messageFails[dest.toB58String()] || 0
    s.messageFails[dest.toB58String()] = prev + 1
    this.set(s)
  }

  /**
   * Reset the number of failed messages sending to a node to zero
   * @param dest Public key of the destination that supposed to received messages but failed.
   */
  resetMessageFails(dest: PublicKey): void {
    const s = this.get()
    s.messageFails[dest.toB58String()] = 0
    this.set(s)
  }
}
