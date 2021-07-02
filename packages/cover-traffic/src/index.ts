//import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, HoprDB, ChannelEntry } from '@hoprnet/hopr-utils'
import { createChainWrapper, Indexer, CONFIRMATIONS, INDEXER_BLOCK_RANGE } from '@hoprnet/hopr-core-ethereum'

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

type State = {
  nodes: any
  channels: Record<string, ChannelEntry>
}

const STATE: State = {
  nodes: {},
  channels: {}
}

const onChannelUpdate = (newChannel) => {
  STATE.channels[newChannel.getId().toHex()] = newChannel
}

const peerUpdate = (peer) => {
  STATE.nodes[PublicKey.fromPeerId(peer.id).toAddress().toHex()] = {
    id: peer.id,
    multiaddrs: peer.multiaddrs,
    pub: PublicKey.fromPeerId(peer.id)
  }
}

async function main() {
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
  console.log('indexing...')
  await indexer.start()
  console.log('done')

  console.log('PEERS')
  Object.values(STATE.nodes)
    .sort((a: any, b: any) => importance(b.pub).cmp(importance(a.pub)))
    .forEach((peer: any) => {
      console.log(
        peer.pub.toAddress().toHex(),
        importance(peer.pub).toString()
        /*totalChannelBalanceFor(peer.pub).toString(),  peer.multiaddrs.map(m => m.toString()).join(',')*/
      )
    })

  /*
  console.log("CHANNELS")
  Object.keys(STATE.channels).forEach((c) => {
    const channel = STATE.channels[c]
    console.log(c, ':', 
      channel.source.toAddress().toHex(), '(' + totalChannelBalanceFor(channel.source).toString() + ')',
      '->',
      channel.destination.toAddress().toHex(), '(' + totalChannelBalanceFor(channel.destination).toString() + ')',
      channel.balance.toFormattedString(), channel.balance.toBN().toString() 
               )
  })
  */
}

main()
