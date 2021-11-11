#!/usr/bin/env node

import BN from 'bn.js'
import yargs from 'yargs/yargs'
import { terminalWidth } from 'yargs'

import { createHoprNode, resolveEnvironment, supportedEnvironments } from '@hoprnet/hopr-core'
import { ChannelEntry, privKeyToPeerId, PublicKey, debug } from '@hoprnet/hopr-utils'

import { PersistedState } from './state'
import { CoverTrafficStrategy } from './strategy'

import type PeerId from 'peer-id'
import type { HoprOptions, ResolvedEnvironment } from '@hoprnet/hopr-core'
import type { PeerData, State } from './state'

const log = debug('hopr:cover-traffic')

function stopGracefully(signal: number) {
  console.log(`Process exiting with signal ${signal}`)
  process.exit()
}

export type DefaultEnvironment = {
  id?: string
}

function defaultEnvironment(): string {
  try {
    const config = require('../default-environment.json') as DefaultEnvironment
    return config?.id || ''
  } catch (error) {
    // its ok if the file isn't there or cannot be read
    return ''
  }
}

const argv = yargs(process.argv.slice(2))
  .option('environment', {
    string: true,
    describe: 'Environment id which the node shall run on',
    choices: supportedEnvironments().map((env) => env.id),
    default: defaultEnvironment()
  })
  .option('privateKey', {
    describe: 'A private key to be used for the node',
    string: true,
    demandOption: true
  })
  .wrap(Math.min(120, terminalWidth()))
  .parseSync()

async function generateNodeOptions(environment: ResolvedEnvironment): Promise<HoprOptions> {
  const options: HoprOptions = {
    announce: false,
    createDbIfNotExist: true,
    environment,
    forceCreateDB: true,
    password: ''
  }

  return options
}

export async function main(update: (State: State) => void, peerId?: PeerId) {
  const environment = resolveEnvironment(argv.environment)
  const options = await generateNodeOptions(environment)
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

  log('creating a node')
  const node = createHoprNode(peerId, options)
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
