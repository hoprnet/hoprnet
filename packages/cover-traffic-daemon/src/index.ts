#!/usr/bin/env node

import path from 'path'

import BN from 'bn.js'
import yargs from 'yargs/yargs'
import { terminalWidth } from 'yargs'
import { createHoprNode, resolveEnvironment, supportedEnvironments, type ResolvedEnvironment } from '@hoprnet/hopr-core'
import { type ChannelEntry, privKeyToPeerId, PublicKey, debug } from '@hoprnet/hopr-utils'

import { PersistedState } from './state'
import { CoverTrafficStrategy } from './strategy'
import setupHealthcheck from './healthcheck'

import type PeerId from 'peer-id'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type { PeerData, State } from './state'

const log = debug('hopr:cover-traffic')
const verbose = debug('hopr:cover-traffic:verbose')

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

// Replace default process name (`node`) by `hopr-cover-traffic-daemon`
process.title = 'hopr-cover-traffic-daemon'

// Use environment-specific default data path
const defaultDataPath = path.join(process.cwd(), 'hopr-cover-traffic-daemon-db', defaultEnvironment())

const argv = yargs(process.argv.slice(2))
  .env('HOPR_CTD') // enable options to be set as environment variables with the HOPR_CTD prefix
  .epilogue(
    'All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.'
  )
  .option('environment', {
    string: true,
    describe: 'Environment id which the node shall run on (HOPR_CTD_ENVIRONMENT)',
    choices: supportedEnvironments().map((env) => env.id),
    default: defaultEnvironment()
  })
  .option('privateKey', {
    describe: 'A private key to be used for the node [env: HOPR_CTD_PRIVATE_KEY]',
    string: true,
    demandOption: true
  })
  .option('provider', {
    string: true,
    describe: 'A custom RPC provider to be used for the node to connect to blockchain [env: HOPR_CTD_PROVIDER]'
  })
  .option('dbFile', {
    describe: 'A path to DB file for persistent storage [env: HOPR_CTD_DB_FILE]',
    string: true,
    default: './ct.json'
  })
  .option('data', {
    string: true,
    describe: 'manually specify the data directory to use [env: HOPR_CTD_DATA]',
    default: defaultDataPath
  })
  .option('healthCheck', {
    boolean: true,
    describe: 'Run a health check end point on localhost:8080 [env: HOPR_CTD_HEALTH_CHECK]',
    default: false
  })
  .option('healthCheckHost', {
    describe: 'Host to listen on for health check [env: HOPR_CTD_HEALTH_CHECK_HOST]',
    default: 'localhost'
  })
  .option('healthCheckPort', {
    describe: 'Port to listen on for health check [env: HOPR_CTD_HEALTH_CHECK_PORT]',
    default: 8080
  })
  .option('allowLocalNodeConnections', {
    boolean: true,
    describe: 'Allow connections to other nodes running on localhost [env: HOPR_CTD_ALLOW_LOCAL_NODE_CONNECTIONS]',
    default: false
  })
  .option('testAnnounceLocalAddresses', {
    boolean: true,
    describe: 'For testing local testnets. Announce local addresses [env: HOPR_CTD_TEST_ANNOUNCE_LOCAL_ADDRESSES]',
    default: false
  })
  .option('testPreferLocalAddresses', {
    boolean: true,
    describe: 'For testing local testnets. Prefer local peers to remote [env: HOPR_CTD_TEST_PREFER_LOCAL_ADDRESSES]',
    default: false
  })
  .wrap(Math.min(120, terminalWidth()))
  .parseSync()

async function generateNodeOptions(environment: ResolvedEnvironment): Promise<HoprOptions> {
  const options: HoprOptions = {
    announce: false,
    createDbIfNotExist: true,
    environment,
    forceCreateDB: false,
    password: '',
    dataPath: argv.data,
    allowLocalConnections: argv.allowLocalNodeConnections,
    testing: {
      announceLocalAddresses: argv.testAnnounceLocalAddresses,
      preferLocalAddresses: argv.testPreferLocalAddresses
    }
  }

  return options
}

export async function main(update: (State: State) => void, peerId?: PeerId) {
  const environment = resolveEnvironment(argv.environment, argv.provider)
  const options = await generateNodeOptions(environment)
  if (!peerId) {
    peerId = privKeyToPeerId(argv.privateKey)
  }

  const selfPub = PublicKey.fromPeerId(peerId)
  const selfAddr = selfPub.toAddress()
  const data = new PersistedState(update, argv.dbFile)

  const onChannelUpdate = (newChannel: ChannelEntry) => {
    data.setChannel(newChannel)
  }

  function logMessageToNode(msg: Uint8Array) {
    log(`Received message ${msg.toString()}`)
  }

  const peerUpdate = (peer: PeerData) => {
    log('adding peer', peer.id.toB58String())
    data.setNode(peer)
  }

  log('creating a node')
  const node = await createHoprNode(peerId, options)

  node.on('hopr:message', logMessageToNode)

  log('setting up indexer')
  node.indexer.on('channel-update', onChannelUpdate)
  node.indexer.on('peer', peerUpdate)
  node.indexer.on('block', (blockNumber) => data.setBlock(new BN(blockNumber.toString())))

  log(`Address: ${selfAddr.toHex()}`)

  log('waiting for node to be funded')
  await node.waitForFunds()

  if (argv.healthCheck) {
    setupHealthcheck(node, argv.healthCheckHost, argv.healthCheckPort)
  }

  log('starting node ...')
  await node.start()
  log('node is running')

  log('hopr-core version: ', node.getVersion())
  log(node.smartContractInfo())

  const channels = await node.getChannelsFrom(selfAddr)
  data.setCTChannels(channels.map((c) => ({ destination: c.destination, latestQualityOf: 0, openFrom: Date.now() })))
  node.setChannelStrategy(new CoverTrafficStrategy(selfPub, node, data))

  setInterval(async () => {
    // CT stats
    verbose('-- CT Stats --')
    verbose(await node.connectionReport())
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
    log(`State update: ${Object.keys(state.nodes).length} nodes, ${Object.keys(state.channels).length} channels`)
  })
}
