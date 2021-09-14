import { PersistedState } from './state'
import type { PeerData, State } from './state'
import type { HoprOptions } from '@hoprnet/hopr-core'
import Hopr from '@hoprnet/hopr-core'
import { CoverTrafficStrategy } from './strategy'
import { ChannelEntry, privKeyToPeerId, PublicKey } from '@hoprnet/hopr-utils'
import type PeerId from 'peer-id'
import BN from 'bn.js'

const priv = process.argv[2]
const peerId = privKeyToPeerId(priv)

function stopGracefully(signal: number) {
  console.log(`Process exiting with signal ${signal}`)
  process.exit()
}

const options: HoprOptions = {
  //provider: 'wss://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/',
  // provider: 'https://eth-goerli.gateway.pokt.network/v1/6021a2b6928ff9002e6c7f2f',
  provider: 'wss://goerli.infura.io/ws/v3/51d4d972f30c4d92b61f2b3898fccaf6',
  createDbIfNotExist: true,
  password: '',
  forceCreateDB: true,
  announce: false
}

export async function main(update: (State: State) => void, peerId: PeerId) {
  const selfPub = PublicKey.fromPeerId(peerId)
  const selfAddr = selfPub.toAddress()
  const data = new PersistedState(update)

  const onChannelUpdate = (newChannel: ChannelEntry) => {
    data.setChannel(newChannel)
  }

  const peerUpdate = (peer: PeerData) => {
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
  const channels = await node.getChannelsFrom(selfAddr, true)
  data.setCTChannels(channels.map((c) => ({ destination: c.destinationPubKey, latestQualityOf: 0 })))
  node.setChannelStrategy(new CoverTrafficStrategy(selfPub, node, data))
}

if (require.main === module) {
  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)
  process.on('uncaughtException', stopGracefully)

  main((_state: State) => {
    console.log('CT: State update')
  }, peerId)
}
