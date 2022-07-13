import BN from 'bn.js'
import { PublicKey, ChannelEntry, ChannelStatus, randomFloat } from '@hoprnet/hopr-utils'
import type { PeerId } from '@libp2p/interface-peer-id'
import { peerIdFromString } from '@libp2p/peer-id'
import type { Multiaddr } from '@multiformats/multiaddr'
import { findChannel, importance } from './utils.js'
import fs from 'fs'

export type ChannelData = {
  channel: ChannelEntry
  // number of attempts of a channel to receive and send CT packets as the 1st hop (aka the recipient of CT channel)
  sendAttempts: number
  // number of attempts of a channel to forward packets (aka as intermediate hops other than the 1st hop).
  forwardAttempts: number
}

export type PeerData = {
  id: PeerId
  pub: PublicKey
  multiaddrs: Multiaddr[]
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

// serialized representation of State type
type SerializedState = {
  nodes: {
    id: string
    multiaddrs: string[]
  }[]
  channels: {
    channel: string
    forwardAttempts: number
    sendAttempts: number
  }[]
  ctChannels: {
    destination: string
    openFrom: number
  }[]
  block: string
}

export function serializeState(state: State): string {
  return JSON.stringify({
    nodes: Object.values(state.nodes).map((node: PeerData) => ({
      // Using hex representation since deserializing Base58 encoded
      // strings is complex
      id: node.id.toString(),
      multiaddrs: node.multiaddrs.map((ma: Multiaddr) => ma.toString())
    })),
    channels: Object.values(state.channels).map((c: ChannelData) => ({
      channel: Buffer.from(c.channel.serialize()).toString('base64'),
      forwardAttempts: c.forwardAttempts,
      sendAttempts: c.sendAttempts
    })),
    ctChannels: state.ctChannels.map((open: OpenChannels) => ({
      destination: open.destination.toCompressedPubKeyHex(),
      openFrom: open.openFrom
    })),
    block: state.block.toString()
  } as SerializedState)
}

export function deserializeState(serialized: string): State {
  const parsed = JSON.parse(serialized) as SerializedState

  return {
    nodes: parsed.nodes.reduce((acc, node) => {
      // Using hex representation since deserializing Base58 encoded
      // strings is complex
      const id = peerIdFromString(node.id)
      acc[id.toString()] = { id, pub: PublicKey.fromPeerId(id), multiaddrs: [] }
      return acc
    }, {}),
    channels: parsed.channels.reduce((acc, c) => {
      const channel = ChannelEntry.deserialize(Uint8Array.from(Buffer.from(c.channel, 'base64')))
      acc[channel.getId().toHex()] = {
        channel,
        forwardAttempts: c.forwardAttempts,
        sendAttempts: c.sendAttempts
      }
      return acc
    }, {}),
    ctChannels: parsed.ctChannels.map((p) => ({
      destination: PublicKey.fromString(p.destination),
      latestQualityOf: 0,
      openFrom: p.openFrom
    })),
    messageFails: {},
    messageTotalSuccess: 0,
    block: new BN(parsed.block)
  }
}

export class PersistedState {
  // Quick and dirty DB.
  // Caveats:
  // - Must live in same timeline as the hoprdb, as it relies on
  //   the indexer being in the same state.
  protected _data: State

  /**
   * Initiate the persisted state of the network attached to the CT node
   * @param update function that is called at every change of the state
   */
  constructor(protected update: (s: State) => void, protected db_path: string) {
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
    const serialized = fs.readFileSync(this.db_path, 'utf8')
    this._data = deserializeState(serialized)
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
    fs.writeFileSync(this.db_path, serializeState(s), 'utf8')
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
    state.nodes[peer.id.toString()] = {
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
    const state = this.get()
    state.block = block
    this.set(state)
  }

  /**
   * Get the PeerData associated with a node from the persisted state
   * @param b58String Node ID encoded in base58
   * @returns PeerData of the given ID.
   */
  getNode(b58String: string): PeerData {
    const state = this.get()
    return state.nodes[b58String]
  }

  /**
   * From all the network channels saved in the persisted state, find the
   * ChannelEntry that is between the provided source and destination.
   * @param src Public key of the `source` of the channel
   * @param dest Public key of the `destination` of the channel
   * @returns ChannelEntry between `source` and `destination`, undefined otherwise.
   */
  findChannel(src: PublicKey, dest: PublicKey): ChannelEntry {
    const state = this.get()
    return findChannel(src, dest, state)
  }

  /**
   * Randomly return a node in the network. The chance of being picked up is
   * in proportion to its weights, which is calculated through the formula of
   * `importance`.
   * @returns a Public key of the node being randomed chosen based on its
   * weight, throw an error otherwise.
   */
  weightedRandomChoice(): PublicKey {
    const state = this.get()
    if (Object.keys(state.nodes).length == 0) {
      throw new Error('no nodes to pick from')
    }
    // key: Public key of a node, value: Importance score of a node
    const weights: Record<string, BN> = {}
    let total = new BN('0')
    const ind = randomFloat()

    // for all the nodes in the network, set importance score as its weight and calculate the sum of all weights
    for (const node of Object.values(state.nodes)) {
      // PublicKey might not be decompressed yet, so using compressed
      // representation makes sure the PublicKey does not get decompressed without need
      const nodeString = node.pub.toCompressedPubKeyHex()
      weights[nodeString] = importance(node.pub, state)
      total = total.add(weights[nodeString])
    }

    if (total.lten(0)) {
      // No important nodes - let's pick a random node for now.
      const index = Math.floor(ind * Object.keys(state.nodes).length)
      return Object.values(state.nodes)[index].pub
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
    const state = this.get()
    const channel = findChannel(source, destination, state)
    if (channel) {
      const channelId = channel.getId().toHex()
      const prev = state.channels[channelId].sendAttempts || 0
      state.channels[channelId].sendAttempts = prev + 1
      this.set(state)
    }
  }

  /**
   * Increase the number of attempts for a channel to forward packets by 1
   * @param source Public key of the message sender.
   * @param destination Public key of the message sender.
   */
  async incrementForwards(source: PublicKey, destination: PublicKey) {
    const state = this.get()
    const channel = findChannel(source, destination, state)
    if (channel) {
      const channelId = channel.getId().toHex()
      const prev = state.channels[channelId].forwardAttempts || 0
      state.channels[channelId].forwardAttempts = prev + 1
      this.set(state)
    }
  }

  /**
   * Returns the number of open channels in the network
   * @returns number of all the open channels
   */
  openChannelCount(): number {
    const state = this.get()
    return Object.values(state.channels).filter((x) => x.channel.status != ChannelStatus.Closed).length
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
    const state = this.get()
    const prev = state.messageTotalSuccess || 0
    state.messageTotalSuccess = prev + 1
    this.set(state)
  }

  /**
   * Get the number of failed messages that should have been sent to the destination
   * @param dest Public key of the destination that supposed to received messages but failed.
   * @returns number of failed messages. If none, returns zero.
   */
  messageFails(dest: PublicKey): number {
    return this.get().messageFails[dest.toString()] || 0
  }

  /**
   * Update the number of failed messages sending to a node
   * @param dest Public key of the destination that supposed to received messages but failed.
   */
  incrementMessageFails(dest: PublicKey): void {
    const state = this.get()
    const prev = state.messageFails[dest.toString()] || 0
    state.messageFails[dest.toString()] = prev + 1
    this.set(state)
  }

  /**
   * Reset the number of failed messages sending to a node to zero
   * @param dest Public key of the destination that supposed to received messages but failed.
   */
  resetMessageFails(dest: PublicKey): void {
    const state = this.get()
    state.messageFails[dest.toString()] = 0
    this.set(state)
  }
}
