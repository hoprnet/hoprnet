//import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, HoprDB, ChannelEntry } from '@hoprnet/hopr-utils'
import { createChainWrapper, Indexer, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from '@hoprnet/hopr-core-ethereum'

import blessed from 'blessed'
import contrib from 'blessed-contrib'

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


type PeerData = {
  id: any, //PeerId,
  pub: PublicKey,
  multiaddrs: any
}
type State = {
  nodes: Record<string, PeerData>
  channels: Record<string, ChannelEntry>
  log: string[]
}

const STATE: State = {
  nodes: {},
  channels: {},
  log: []
}

function setupDashboard() {
  const screen = blessed.screen()
  const grid = new contrib.grid({rows: 4, cols: 2, screen: screen})
  screen.key(['escape', 'q', 'C-c'], function() {
    return process.exit(0);
  });

  const table = grid.set(0, 0, 3, 2, contrib.table, {
      fg: 'white', label: 'Nodes'
    , keys: true
    , interactive: true
     , border: {type: "line", fg: "cyan"}
     , columnSpacing: 10 //in chars
     , columnWidth: [50, 40, 12] /*in chars*/ } as any)
   table.focus()

  const logs = grid.set(3,0, 1, 2, contrib.log, {})

  screen.render()

  const update = () => {
    table.setData(
     { headers: ['ID', 'Address', 'Importance']
     , data: Object.values(STATE.nodes)
              .sort((a: any, b: any) => importance(b.pub).cmp(importance(a.pub)))
               .map(p => [
        p.id.toB58String(), p.pub.toAddress().toHex(),
        new BigNumber(importance(p.pub).toString()).toPrecision(4, 0)
     ])
    })

    var l
    while (l = STATE.log.pop()){
      logs.log(l)
    }

    screen.render()
  }
  update()

  return update
}

async function main() {
  const update = setupDashboard()

  const onChannelUpdate = (newChannel) => {
    STATE.channels[newChannel.getId().toHex()] = newChannel
    update()
  }

  const peerUpdate = (peer) => {
    STATE.nodes[PublicKey.fromPeerId(peer.id).toAddress().toHex()] = {
      id: peer.id,
      multiaddrs: peer.multiaddrs,
      pub: PublicKey.fromPeerId(peer.id)
    }
    update()
  }

  const priv = process.argv[2]
  const peerId = privKeyToPeerId(priv)
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
  await indexer.start()
  STATE.log.push('done')
  update()
}

main()
