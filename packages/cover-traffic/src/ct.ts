import type { HoprOptions, ChannelsToOpen, ChannelsToClose } from '@hoprnet/hopr-core'
import Hopr, { SaneDefaults, findPath} from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, HoprDB, ChannelEntry } from '@hoprnet/hopr-utils'
import { createChainWrapper, Indexer, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from '@hoprnet/hopr-core-ethereum'
import type PeerId from 'peer-id'

const CHANNELS_PER_COVER_TRAFFIC_NODE = 5
const CHANNEL_STAKE = new BN('1000')
const MINIMUM_STAKE_BEFORE_CLOSURE = new BN('0')
const CT_INTERMEDIATE_HOPS = 3 // NB. min is 2

const options: HoprOptions = {
  //provider: 'wss://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/',
  // provider: 'https://eth-goerli.gateway.pokt.network/v1/6021a2b6928ff9002e6c7f2f',
  provider: 'wss://goerli.infura.io/ws/v3/51d4d972f30c4d92b61f2b3898fccaf6',
  createDbIfNotExist: true,
  password: '',
  forceCreateDB: true,
  announce: false
}

export const addBN = (a: BN, b: BN): BN => a.add(b)
export const sqrtBN = (a: BN): BN => new BN(new BigNumber(a.toString()).squareRoot().toString())
export const findChannelsFrom = (p: PublicKey): ChannelEntry[] =>
  Object.values(STATE.channels).filter((c: ChannelEntry) => c.source.eq(p))
export const totalChannelBalanceFor = (p: PublicKey): BN =>
  findChannelsFrom(p)
    .map((c) => c.balance.toBN())
    .reduce(addBN, new BN('0'))

export const importance = (p: PublicKey): BN =>
  findChannelsFrom(p)
    .map((c: ChannelEntry) =>
      sqrtBN(totalChannelBalanceFor(p).mul(c.balance.toBN()).mul(totalChannelBalanceFor(c.destination)))
    )
    .reduce(addBN, new BN('0'))

export const findChannel = (src: PublicKey, dest: PublicKey): ChannelEntry =>
  Object.values(STATE.channels).find((c: ChannelEntry) => c.source.eq(src) && c.destination.eq(dest))
export const getNode = (b58String: string): PeerData => STATE.nodes[b58String]

export const weightedRandomChoice = (): PublicKey => {
  if (Object.keys(STATE.nodes).length == 0) {
    throw new Error('no nodes to pick from')
  }
  const weights: Record<string, BN> = {}
  let total = new BN('0')
  Object.values(STATE.nodes).forEach((p) => {
    weights[p.pub.toHex()] = importance(p.pub)
    total = total.add(weights[p.pub.toHex()])
  })

  const ind = Math.random()
  let interval = total.muln(ind)
  for (let node of Object.keys(weights)) {
    interval = interval.sub(weights[node])
    if (interval.lte(new BN('0'))) {
      return PublicKey.fromString(node)
    }
  }
  throw new Error('wtf')
}

export const sendCTMessage = async (startNode: PublicKey, selfPub: PublicKey): Promise<boolean> => {
  const weight = (edge: ChannelEntry): BN => importance(edge.destination)
  try {
    const path = await findPath(
      startNode,
      selfPub,
      CT_INTERMEDIATE_HOPS - 1,// As us to start is first intermediate
      (_p: PublicKey): number => 1, // TODO network quality?
      (p: PublicKey) => Promise.resolve(findChannelsFrom(p)),
      weight
    )
    path.push(selfPub) // destination is always self.
    STATE.log.push('SEND ' + path.map(pub => pub.toPeerId().toB58String()).join(','))
  } catch (e) {
    // could not find path
    STATE.log.push('Could not find path - ' + startNode.toPeerId().toB58String())
    return false
  }
  // TODO _send_
  return true
}

class CoverTrafficStrategy extends SaneDefaults {
  name = 'covertraffic'
  constructor(private update: (State) => void, private selfPub: PublicKey) {
    super()
  }

  tickInterval = 10000

  async tick(
    balance: BN,
    currentChannels: ChannelEntry[],
    _peers: any,
    _getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    const toOpen = []
    const toClose = []

    currentChannels.forEach((curr) => {
      if (curr.balance.toBN().lte(MINIMUM_STAKE_BEFORE_CLOSURE)) {
        toClose.push(curr.destination)
      }
    })

    if (currentChannels.length < CHANNELS_PER_COVER_TRAFFIC_NODE && Object.keys(STATE.nodes).length > 0) {
      const c = weightedRandomChoice()
      if (!currentChannels.find((x) => x.destination.eq(c)) && !c.eq(this.selfPub)) {
        toOpen.push([c, CHANNEL_STAKE])
      }
    }

    STATE.ctChannels = currentChannels
      .map((c) => c.destination)
      .concat(toOpen.map((o) => o[0]))
      .concat(toClose)
    STATE.log.push(
      (`strategy tick: balance:${balance.toString()
       } open:${toOpen.map((p) => p[0].toPeerId().toB58String()).join(',')
       } close: ${toClose
        .map((p) => p.toPeerId().toB58String())
        .join(',')}`
    ).replace('\n', ', '))

    await Promise.all(STATE.ctChannels.map(async (dest) => {
      const success = await sendCTMessage(dest, this.selfPub)
      if (!success) {
        toClose.push(dest);
      }
    }))

    this.update(STATE)
    return [toOpen, toClose]
  }
}


type PeerData = {
  id: any //PeerId,
  pub: PublicKey
  multiaddrs: any
}
export type State = {
  nodes: Record<string, PeerData>
  channels: Record<string, ChannelEntry>
  log: string[]
  ctChannels: PublicKey[]
  block: BN
}

const STATE: State = {
  nodes: {},
  channels: {},
  log: [],
  ctChannels: [],
  block: new BN('0')
}

/*
// Otherwise we get a mess
process.on('unhandledRejection', (_reason, promise) => {
  STATE.log.push('uncaught exception in promise' + promise)
})
*/

export async function main(update: (State) => void, peerId: PeerId) {
  const selfPub = PublicKey.fromPeerId(peerId)
  const selfAddr = selfPub.toAddress()

  const onChannelUpdate = (newChannel) => {
    STATE.channels[newChannel.getId().toHex()] = newChannel
    update(STATE)
  }

  const peerUpdate = (peer) => {
    STATE.nodes[peer.id.toB58String()] = {
      id: peer.id,
      multiaddrs: peer.multiaddrs,
      pub: PublicKey.fromPeerId(peer.id)
    }
    update(STATE)
  }

  const db = new HoprDB(
    PublicKey.fromPrivKey(peerId.privKey.marshal()).toAddress(),
    options.createDbIfNotExist,
    'cover-traffic',
    options.dbPath,
    options.forceCreateDB
  )

  const chain = await createChainWrapper(options.provider, peerId.privKey.marshal())
  await chain.waitUntilReady()
  const indexer = new Indexer(chain.getGenesisBlock(), db, chain, CONFIRMATIONS, INDEXER_BLOCK_RANGE)
  indexer.on('channel-update', onChannelUpdate)
  indexer.on('peer', peerUpdate)
  indexer.on('block', (blockNumber) => {
    STATE.block = new BN(blockNumber.toString())
    update(STATE)
  })
  STATE.log.push('indexing...')
  update(STATE)
  await indexer.start()
  STATE.log.push('done')
  update(STATE)
  STATE.log.push('creating a node...')
  update(STATE)
  const node = new Hopr(peerId, options)
  STATE.log.push('waiting for node to be funded ...')
  update(STATE)
  await node.waitForFunds()
  STATE.log.push('starting node ...')
  update(STATE)
  await node.start()
  STATE.log.push('node is running')
  const channels = await node.getChannelsFrom(selfAddr)
  channels.forEach((c) => STATE.ctChannels.push(c.destination))
  update(STATE)
  node.setChannelStrategy(new CoverTrafficStrategy(update, selfPub))
}
