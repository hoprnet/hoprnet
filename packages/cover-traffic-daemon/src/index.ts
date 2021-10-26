#!/usr/bin/env node
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
import { debug } from '@hoprnet/hopr-utils'

const log = debug('hopr:cover-traffic')

function stopGracefully(signal: number) {
  console.log(`Process exiting with signal ${signal}`)
  process.exit()
}

const argv = yargs(process.argv.slice(2))
  .option('provider', {
    describe: 'A provider url for the network this node shall operate on',
    default: 'https://provider-proxy.hoprnet.workers.dev/matic_rio',
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

  log('creating a node...')
  const node = new Hopr(peerId, options)
  log('setting up indexer')
  node.indexer.on('channel-update', onChannelUpdate)
  node.indexer.on('peer', peerUpdate)
  node.indexer.on('block', (blockNumber) => data.setBlock(new BN(blockNumber.toString())))

  log('waiting for node to be funded')
  await node.waitForFunds()
  log('starting node ...')
  await node.start()
  log('node is running')

  console.log(node.getVersion())
  console.log(node.smartContractInfo())

  const channels = await node.getChannelsFrom(selfAddr)
  data.setCTChannels(channels.map((c) => ({ destination: c.destination, latestQualityOf: 0, openFrom: Date.now() })))
  node.setChannelStrategy(new CoverTrafficStrategy(selfPub, node, data))

  setInterval(async () => {
    // CT stats
    console.log('-- CT Stats --')
    console.log(await node.connectionReport())
  }, 5000)
}

if (require.main === module) {
  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)

  process.on('uncaughtExceptionMonitor', (err, origin) => {
    // Make sure we get a log.
    log(`FATAL ERROR, exiting with uncaught exception: ${origin} ${err}`)
  })

  main((state: State) => {
    console.log(
      `CT: State update:` +
        `${Object.keys(state.nodes).length} nodes, ` +
        `${Object.keys(state.channels).length} channels`
    )
  })
}
