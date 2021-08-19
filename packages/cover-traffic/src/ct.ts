import type { HoprOptions, ChannelsToOpen, ChannelsToClose } from '@hoprnet/hopr-core'
import Hopr, { SaneDefaults, findPath } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, ChannelEntry, ChannelStatus } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import fs from 'fs'

const CHANNELS_PER_COVER_TRAFFIC_NODE = 10
const CHANNEL_STAKE = new BN('1000')
const MINIMUM_STAKE_BEFORE_CLOSURE = new BN('0')
const CT_INTERMEDIATE_HOPS = 2 // 3  // NB. min is 2
const DB = './ct.json'
const MESSAGE_FAIL_THRESHOLD = 1000 // Failed sends to channel before we autoclose

const options: HoprOptions = {
  //provider: 'wss://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/',
  // provider: 'https://eth-goerli.gateway.pokt.network/v1/6021a2b6928ff9002e6c7f2f',
  provider: 'wss://goerli.infura.io/ws/v3/51d4d972f30c4d92b61f2b3898fccaf6',
  createDbIfNotExist: true,
  password: '',
  forceCreateDB: true,
  announce: false
}

type PeerData = {
  id: any //PeerId,
  pub: PublicKey
  multiaddrs: any
}

export type ChannelData = {
  channel: ChannelEntry
  sendAttempts: number
  forwardAttempts: number
}

export type OpenChannels = {
  destination: PublicKey
  latestQualityOf: number
}

export type State = {
  nodes: Record<string, PeerData>
  channels: Record<string, ChannelData>
  log: string[]
  ctChannels: OpenChannels[]
  block: BN
  messageFails: Record<string, number>
}

class PersistedState {
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

  async load(): Promise<void> {
    const json = JSON.parse(fs.readFileSync(DB, 'utf8'))
    this._data = {
      nodes: {},
      channels: {},
      log: ['loaded data'],
      ctChannels: json.ctChannels.map((p) => ({
        destination: PublicKey.fromPeerId(PeerId.createFromB58String(p)),
        latestQualityOf: 0
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

  async get(): Promise<State> {
    return this._data
  }

  async set(s: State) {
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
        ctChannels: s.ctChannels.map((o: OpenChannels) => o.destination.toB58String()),
        block: s.block.toString()
      }),
      'utf8'
    )
    this.update(s)
    return
  }

  async setChannel(channel: ChannelEntry) {
    const state = await this.get()
    if (state.channels[channel.getId().toHex()]) {
      state.channels[channel.getId().toHex()].channel = channel
    } else {
      state.channels[channel.getId().toHex()] = {
        channel,
        sendAttempts: 0,
        forwardAttempts: 0
      }
    }
    await this.set(state)
  }

  async setNode(peer) {
    const state = await this.get()
    state.nodes[peer.id.toB58String()] = {
      id: peer.id,
      multiaddrs: peer.multiaddrs,
      pub: PublicKey.fromPeerId(peer.id)
    }
    await this.set(state)
  }

  async setCTChannels(channels: OpenChannels[]) {
    const state = await this.get()
    state.ctChannels = channels
    await this.set(state)
  }

  async findChannelsFrom(p: PublicKey): Promise<ChannelEntry[]> {
    return Object.values((await this.get()).channels)
      .filter((c: ChannelData) => c.channel.source.eq(p))
      .map((c) => c.channel)
  }

  async log(...args: String[]) {
    const s = await this.get()
    s.log.push(args.join(' '))
    await this.set(s)
  }

  async setBlock(block: BN) {
    const s = await this.get()
    s.block = block
    await this.set(s)
  }

  async getNode(b58String: string): Promise<PeerData> {
    const s = await this.get()
    return s.nodes[b58String]
  }

  async findChannel(src: PublicKey, dest: PublicKey): Promise<ChannelEntry> {
    const s = await this.get()
    return findChannel(src, dest, s)
  }

  async weightedRandomChoice(): Promise<PublicKey> {
    const s = await this.get()
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

  async openChannelCount(): Promise<number> {
    const s = await this.get()
    return Object.values(s.channels).filter((x) => x.channel.status != ChannelStatus.Closed).length
  }

  async messageFails(dest: PublicKey): Promise<number> {
    return (await this.get()).messageFails[dest.toB58String()] || 0
  }

  async incrementMessageFails(dest: PublicKey) {
    const s = await this.get()
    const prev = s.messageFails[dest.toB58String()] || 0
    s.messageFails[dest.toB58String()] = prev + 1
    await this.set(s)
  }

  async resetMessageFails(dest: PublicKey) {
    const s = await this.get()
    s.messageFails[dest.toB58String()] = 0
    await this.set(s)
  }
}

export const addBN = (a: BN, b: BN): BN => a.add(b)
export const sqrtBN = (a: BN): BN => new BN(new BigNumber(a.toString()).squareRoot().integerValue().toFixed(), 10)

export const findChannelsFrom = (p: PublicKey, state: State): ChannelEntry[] =>
  Object.values(state.channels)
    .map((c) => c.channel)
    .filter((c: ChannelEntry) => c.source.eq(p))

export const totalChannelBalanceFor = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c) => c.balance.toBN())
    .reduce(addBN, new BN('0'))

export const importance = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c: ChannelEntry) =>
      sqrtBN(totalChannelBalanceFor(p, state).mul(c.balance.toBN()).mul(totalChannelBalanceFor(c.destination, state)))
    )
    .reduce(addBN, new BN('0'))

export const findChannel = (src: PublicKey, dest: PublicKey, state: State): ChannelEntry =>
  Object.values(state.channels)
    .map((c: ChannelData): ChannelEntry => c.channel)
    .find((c: ChannelEntry) => c.source.eq(src) && c.destination.eq(dest))

export const sendCTMessage = async (
  startNode: PublicKey,
  selfPub: PublicKey,
  sendMessage: (path: PublicKey[]) => Promise<void>,
  data: PersistedState
): Promise<boolean> => {
  const weight = async (edge: ChannelEntry): Promise<BN> => await importance(edge.destination, await data.get())
  let path
  try {
    path = await findPath(
      startNode,
      selfPub,
      CT_INTERMEDIATE_HOPS - 1, // As us to start is first intermediate
      (_p: PublicKey): number => 1, // TODO network quality?
      (p: PublicKey) => data.findChannelsFrom(p),
      weight
    )

    path.forEach((p) => data.incrementForwards(p))
    path.push(selfPub) // destination is always self.
    data.log('SEND ' + path.map((pub) => pub.toB58String()).join(','))
  } catch (e) {
    // could not find path
    data.log(`Could not find path: ${startNode.toB58String()} -> ${selfPub.toPeerId()} (${e})`)
    return false
  }
  try {
    data.incrementSent(startNode)
    await sendMessage(path)
    return true
  } catch (e) {
    //console.log(e)
    data.log('error sending to' + startNode.toPeerId().toB58String())
    return false
  }
}

class CoverTrafficStrategy extends SaneDefaults {
  name = 'covertraffic'
  constructor(private selfPub: PublicKey, private node: Hopr, private data: PersistedState) {
    super()
  }

  tickInterval = 10000

  async tick(
    balance: BN,
    currentChannels: ChannelEntry[],
    peers: any,
    _getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    const toOpen = []
    const toClose = []
    const state = await this.data.get()

    // Refresh open channels
    const ctChannels = []
    for (let c of currentChannels) {
      if (c.status === ChannelStatus.Closed) {
        continue
      }
      const q = await peers.qualityOf(c.destination)
      ctChannels.push({ destination: c.destination, latestQualityOf: q })
      if (q < 0.15) {
        toClose.push(c.destination)
      }
      if (c.balance.toBN().lte(MINIMUM_STAKE_BEFORE_CLOSURE)) {
        toClose.push(c.destination)
      }
      if ((await this.data.messageFails(c.destination)) > MESSAGE_FAIL_THRESHOLD) {
        await this.data.resetMessageFails(c.destination)
        toClose.push(c.destination)
      }
    }
    await this.data.setCTChannels(ctChannels)

    if ((await this.data.openChannelCount()) > CT_INTERMEDIATE_HOPS + 1) {
      for (let openChannel of state.ctChannels) {
        const channel = await this.data.findChannel(this.selfPub, openChannel.destination)
        if (channel && channel.status == ChannelStatus.Open) {
          const success = await sendCTMessage(
            openChannel.destination,
            this.selfPub,
            async (path: PublicKey[]) => {
              await this.node.sendMessage(new Uint8Array(1), openChannel.destination.toPeerId(), path)
            },
            this.data
          )
          if (!success) {
            await this.data.incrementMessageFails(openChannel.destination)
          }
        }
        // TODO handle waiting for commitment stalls
      }
    } else {
      this.data.log('aborting send messages - less channels in network than hops required')
    }

    let attempts = 0
    while (
      currentChannels.length < CHANNELS_PER_COVER_TRAFFIC_NODE &&
      Object.keys(state.nodes).length > 0 &&
      attempts < 100
    ) {
      attempts++
      const c = await this.data.weightedRandomChoice()
      const q = await peers.qualityOf(c)

      if (
        currentChannels.filter((x) => x.status !== ChannelStatus.Closed).find((x) => x.destination.eq(c)) ||
        c.eq(this.selfPub) ||
        toOpen.find((x) => x[1].eq(c))
      ) {
        console.error('skipping node', c.toB58String())
        continue
      }

      if (q < 0.6) {
        console.error('low quality node skipped', c.toB58String(), q)
        continue
      }

      toOpen.push([c, CHANNEL_STAKE])
    }

    this.data.log(
      `strategy tick: ${Date.now()} balance:${balance.toString()} open:${toOpen
        .map((p) => p[0].toPeerId().toB58String())
        .join(',')} close: ${toClose.map((p) => p.toPeerId().toB58String()).join(',')}`.replace('\n', ', ')
    )
    return [toOpen, toClose]
  }
}

export async function main(update: (State) => void, peerId: PeerId) {
  const selfPub = PublicKey.fromPeerId(peerId)
  const selfAddr = selfPub.toAddress()
  const data = new PersistedState(update)

  const onChannelUpdate = (newChannel) => {
    data.setChannel(newChannel)
  }

  const peerUpdate = (peer) => {
    data.setNode(peer)
  }

  data.log('creating a node...')
  const node = new Hopr(peerId, options)
  data.log('setting up indexer')
  node.indexer.on('channel-update', onChannelUpdate)
  node.indexer.on('peer', peerUpdate)
  node.indexer.on('block', (blockNumber) => data.setBlock(new BN(blockNumber.toString())))

  data.log('waiting for node to be funded')
  await node.waitForFunds()
  data.log('starting node ...')
  await node.start()
  data.log('node is running')
  const channels = await node.getChannelsFrom(selfAddr)
  data.setCTChannels(channels.map((c) => ({ destination: c.destination, latestQualityOf: 0 })))
  node.setChannelStrategy(new CoverTrafficStrategy(selfPub, node, data))
}
