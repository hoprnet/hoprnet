#!/usr/bin/env node

import path from 'path'

import BN from 'bn.js'
import yargs from 'yargs/yargs'
import { hideBin } from 'yargs/helpers'
import {
  createHoprNode,
  resolveNetwork,
  supportedNetworks,
  type ResolvedNetwork,
  CONSTANTS
} from '@hoprnet/hopr-core'

import {
  type ChannelEntry,
  privKeyToPeerId,
  PublicKey,
  debug,
  loadJson,
  get_package_version,
  setupPromiseRejectionFilter
} from '@hoprnet/hopr-utils'

import { PersistedState } from './state.js'
import { CoverTrafficStrategy } from './strategy.js'
import setupHealthcheck from './healthcheck.js'

import type { PeerId } from '@libp2p/interface-peer-id'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type { PeerData, State } from './state.js'

const log = debug('hopr:cover-traffic')
const verbose = debug('hopr:cover-traffic:verbose')

function stopGracefully(signal: number) {
  console.log(`Process exiting with signal ${signal}`)
  process.exit()
}

export type DefaultNetwork = {
  id?: string
}

function defaultNetwork(): string {
  try {
    // Don't do typechecks on JSON files
    // @ts-ignore
    const config = loadJson('../default-network.json') as DefaultNetwork
    return config?.id || ''
  } catch (error) {
    // its ok if the file isn't there or cannot be read
    return ''
  }
}

// Use network-specific default data path
const defaultDataPath = path.join(process.cwd(), 'hopr-cover-traffic-daemon-db', defaultNetwork())

// reading the version manually to ensure the path is read correctly
const packageFile = path.normalize(new URL('../package.json', import.meta.url).pathname)
const version = get_package_version(packageFile)

const yargsInstance = yargs(hideBin(process.argv))

// Exported from Rust
const constants = CONSTANTS()

const argv = yargsInstance
  .env('HOPR_CTD') // enable options to be set as environment variables with the HOPR_CTD prefix
  .epilogue(
    'All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.'
  )
  .version(version)
  .option('network', {
    string: true,
    describe: 'Network id which the node shall run on (HOPR_CTD_ENVIRONMENT)',
    choices: supportedNetworks().map((env) => env.id),
    default: defaultNetwork()
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
  .option('heartbeatInterval', {
    number: true,
    describe:
      'Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL]',
    default: constants.DEFAULT_HEARTBEAT_INTERVAL
  })
  .option('heartbeatThreshold', {
    number: true,
    describe:
      "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD]",
    default: constants.DEFAULT_HEARTBEAT_THRESHOLD
  })
  .option('heartbeatVariance', {
    number: true,
    describe: 'Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE]',
    default: constants.DEFAULT_HEARTBEAT_INTERVAL_VARIANCE
  })
  .wrap(Math.min(120, yargsInstance.terminalWidth()))
  .parseSync()

function generateNodeOptions(environment: ResolvedNetwork): HoprOptions {
  const options: HoprOptions = {
    announce: false,
    createDbIfNotExist: true,
    environment,
    forceCreateDB: false,
    password: '',
    dataPath: argv.data,
    allowLocalConnections: argv.allowLocalNodeConnections,
    heartbeatInterval: argv.heartbeatInterval,
    heartbeatThreshold: argv.heartbeatThreshold,
    heartbeatVariance: argv.heartbeatVariance,
    testing: {
      announceLocalAddresses: argv.testAnnounceLocalAddresses,
      preferLocalAddresses: argv.testPreferLocalAddresses
    }
  }

  return options
}

export async function main(update: (State: State) => void, peerId?: PeerId) {
  const environment = resolveNetwork(argv.environment, argv.provider)
  const options = generateNodeOptions(environment)
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
    log(`Received message ${new TextDecoder().decode(msg)}`)
  }

  const peerUpdate = (peer: PeerData) => {
    log('adding peer', peer.id.toString())
    data.setNode(peer)
  }

  log(`This is hopr-cover-traffic-daemon version ${version}`)

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

if (import.meta.url === `file://${process.argv[1]}`) {
  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)

  // Filter specific known promise rejection that cannot be handled for
  // one reason or the other
  setupPromiseRejectionFilter()

  process.on('uncaughtExceptionMonitor', (err, origin) => {
    // Make sure we get a log.
    log(`FATAL ERROR, exiting with uncaught exception:`, origin, err)
  })

  main((state: State) => {
    log(`State update: ${Object.keys(state.nodes).length} nodes, ${Object.keys(state.channels).length} channels`)
  })
}
