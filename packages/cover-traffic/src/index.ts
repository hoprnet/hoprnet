//import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions, ChannelsToOpen, ChannelsToClose } from '@hoprnet/hopr-core'
import Hopr, { SaneDefaults } from '@hoprnet/hopr-core'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, HoprDB, ChannelEntry } from '@hoprnet/hopr-utils'
import { createChainWrapper, Indexer, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from '@hoprnet/hopr-core-ethereum'

import blessed from 'blessed'
import contrib from 'blessed-contrib'

const CHANNELS_PER_COVER_TRAFFIC_NODE = 5
const CHANNEL_STAKE = new BN('1000')

const addBN = (a: BN, b: BN): BN => a.add(b)
const sqrtBN = (a: BN): BN => new BN(new BigNumber(a.toString()).squareRoot().toString())
const findChannelsFrom = (p: PublicKey): ChannelEntry[] =>
  Object.values(STATE.channels).filter((c: ChannelEntry) => c.source.eq(p))
const totalChannelBalanceFor = (p: PublicKey): BN =>
  findChannelsFrom(p)
    .map((c) => c.balance.toBN())
    .reduce(addBN, new BN('0'))

const importance = (p: PublicKey): BN =>
  findChannelsFrom(p)
    .map((c: ChannelEntry) =>
      sqrtBN(totalChannelBalanceFor(p).mul(c.balance.toBN()).mul(totalChannelBalanceFor(c.destination)))
    )
    .reduce(addBN, new BN('0'))

const findChannel = (src: PublicKey, dest: PublicKey): ChannelEntry =>
  Object.values(STATE.channels).find((c: ChannelEntry) => c.source.eq(src) && c.destination.eq(dest))

const weightedRandomChoice = (): PublicKey => {
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


class CoverTrafficStrategy extends SaneDefaults {
  name = "covertraffic"
  constructor(private update) {
    super()
  }

  async tick(
    _balance: BN,
    _n: ChannelEntry[],
    _currentChannels: ChannelEntry[],
    _peers: any,
    _getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    this.update()
    return [[], []]
  }

}

type PeerData = {
  id: any //PeerId,
  pub: PublicKey
  multiaddrs: any
}
type State = {
  nodes: Record<string, PeerData>
  channels: Record<string, ChannelEntry>
  log: string[]
  ctChannels: PublicKey[]
}

const STATE: State = {
  nodes: {},
  channels: {},
  log: [],
  ctChannels: []
}

function setupDashboard(selfPub: PublicKey) {
  const screen = blessed.screen()
  const grid = new contrib.grid({ rows: 4, cols: 4, screen: screen })
  screen.key(['escape', 'q', 'C-c'], function () {
    return process.exit(0)
  })

  const table = grid.set(0, 0, 3, 2, contrib.table, {
    fg: 'white',
    label: 'Nodes',
    keys: true,
    interactive: true,
    border: { type: 'line', fg: 'cyan' },
    columnSpacing: 2,
    columnWidth: [55, 12, 6, 12] /*in chars*/
  } as any)
  table.focus()

  const inspect = grid.set(0, 2, 2, 2, contrib.table, {
    fg: 'white',
    label: 'Selected',
    keys: false,
    interactive: false,
    border: { type: 'line', fg: 'cyan' },
    columnSpacing: 2, //in chars
    columnWidth: [6, 90] /*in chars*/
  } as any)

  const logs = grid.set(3, 0, 1, 4, contrib.log, { label: 'logs' })

  const ctChan = grid.set(2, 2, 1, 2, contrib.table, {
    label: 'Cover Traffic channels',
    columnWidth: [60, 20]
  })

  table.rows.on('select item', (item) => {
    const id = item.content.split(' ')[0].trim()
    const node = STATE.nodes[id]
    if (node) {
      const data = [
        ['id', node.id.toB58String()],
        ['pubkey', node.pub.toHex()],
        ['addr', node.pub.toAddress().toHex()],
        ['ma', node.multiaddrs.map((x) => x.toString()).join(',')]
      ]
      findChannelsFrom(node.pub).forEach((c, i) => {
        data.push([
          'ch.' + i,
          c.destination.toPeerId().toB58String() + ' ' + c.balance.toFormattedString() + ' - ' + c.status
        ])
      })

      inspect.setData({ headers: ['', ''], data })
    }
  })

  screen.render()

  const update = () => {
    table.setData({
      headers: ['ID', 'Importance', '#Chans', 'Tot.Stk'],
      data: Object.values(STATE.nodes)
        .sort((a: any, b: any) => importance(b.pub).cmp(importance(a.pub)))
        .map((p) => [
          p.id.toB58String(),
          new BigNumber(importance(p.pub).toString()).toPrecision(4, 0),
          findChannelsFrom(p.pub).length,
          new BigNumber(totalChannelBalanceFor(p.pub).toString()).toPrecision(4, 0)
        ])
    })

    var l
    while ((l = STATE.log.pop())) {
      logs.log(l)
    }

    ctChan.setData({
      headers: ['Dest', 'Status'],
      data: STATE.ctChannels.map((p: PublicKey) => {
        const chan = findChannel(selfPub, p)
        let status = 'PENDING'
        if (chan) {
          status = chan.status.toString()
        }
        return [p.toPeerId().toB58String(), status]
      })
    })

    screen.render()
  }
  update()

  return update
}

async function tick(update, node: Hopr) {
  if (STATE.ctChannels.length < CHANNELS_PER_COVER_TRAFFIC_NODE && Object.keys(STATE.nodes).length > 0) {
    const toOpen = weightedRandomChoice()
    if (!STATE.ctChannels.find((x) => x.eq(toOpen))) {
      STATE.ctChannels.push(toOpen)
      await node.openChannel(toOpen.toPeerId(), CHANNEL_STAKE) 
    }
  }
  update()
  setTimeout(() => tick(update, node), 1000)
}

async function main() {
  const priv = process.argv[2]
  const peerId = privKeyToPeerId(priv)
  const selfPub = PublicKey.fromPeerId(peerId)
  const selfAddr = selfPub.toAddress()

  const update = setupDashboard(selfPub)

  const onChannelUpdate = (newChannel) => {
    STATE.channels[newChannel.getId().toHex()] = newChannel
    update()
  }

  const peerUpdate = (peer) => {
    STATE.nodes[peer.id.toB58String()] = {
      id: peer.id,
      multiaddrs: peer.multiaddrs,
      pub: PublicKey.fromPeerId(peer.id)
    }
    update()
  }

  const options: HoprOptions = {
    provider: 'wss://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/',
    createDbIfNotExist: true,
    password: '',
    forceCreateDB: true,
    announce: false
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
  STATE.log.push('indexing...')
  update()
  await indexer.start()
  STATE.log.push('done')
  update()
  STATE.log.push('creating a node...')
  update()
  const node = new Hopr(peerId, options)
  STATE.log.push('waiting for node to be funded ...')
  update()
  await node.waitForFunds()
  STATE.log.push('starting node ...')
  update()
  await node.start()
  STATE.log.push('node is running')
  const channels = await node.getChannelsFrom(selfAddr)
  channels.forEach((c) => STATE.ctChannels.push(c.destination))
  update()
  node.setChannelStrategy(new CoverTrafficStrategy(update))
  tick(update, node)
}

main()
