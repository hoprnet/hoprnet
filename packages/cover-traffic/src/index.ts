import { PersistedState } from './state'
import type { PeerData, State } from './state'
import type { HoprOptions } from '@hoprnet/hopr-core'
import Hopr from '@hoprnet/hopr-core'
import yargs from 'yargs/yargs'
import { terminalWidth } from 'yargs'
import { CoverTrafficStrategy } from './strategy'
import { ChannelEntry, privKeyToPeerId, PublicKey } from '@hoprnet/hopr-utils'
import type PeerId from 'peer-id'
import BN from 'bn.js'

function stopGracefully(signal: number) {
  console.log(`Process exiting with signal ${signal}`)
  process.exit()
}

const argv = yargs(process.argv.slice(2))
  .option('provider', {
    describe: 'A provider url for the network this node shall operate on',
    default: 'https://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/',
    string: true
  })
  .option('privateKey', {
    describe: 'A private key to be used for the node',
    string: true,
    demandOption: true
  })
  .wrap(Math.min(120, terminalWidth()))
  .parseSync()

async function generateNodeOptions(): Promise<HoprOptions> {
  const options: HoprOptions = {
  provider: argv.provider,
  createDbIfNotExist: true,
  password: '',
  forceCreateDB: true,
  announce: false
}

  return options
}

export async function main(update: (State: State) => void, peerId?: PeerId) {
  const options = await generateNodeOptions()
  if (!peerId) {
    peerId = privKeyToPeerId(argv.privateKey)
  }
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
  const channels = await node.getChannelsFrom(selfAddr)
  data.setCTChannels(channels.map((c) => ({ destination: c.destination, latestQualityOf: 0 })))
  node.setChannelStrategy(new CoverTrafficStrategy(selfPub, node, data))
}

if (require.main === module) {
  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)
  process.on('uncaughtException', stopGracefully)

  main((_state: State) => {
    console.log('CT: State update')
  })
}
