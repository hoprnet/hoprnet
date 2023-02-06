import path from 'path'

import {
  create_gauge,
  create_multi_gauge,
  get_package_version,
  NativeBalance,
  setupPromiseRejectionFilter,
  SUGGESTED_NATIVE_BALANCE,
  create_histogram_with_buckets,
  pickVersion
} from '@hoprnet/hopr-utils'
import {
  createHoprNode,
  default as Hopr,
  type HoprOptions,
  isStrategy,
  NetworkHealthIndicator,
  ResolvedEnvironment,
  resolveEnvironment
} from '@hoprnet/hopr-core'

import { parse_cli_arguments, type CliArgs } from '../lib/hoprd_misc.js'
import type { State } from './types.js'
import setupAPI from './api/index.js'
import setupHealthcheck from './healthcheck.js'
import { LogStream } from './logs.js'
import { getIdentity } from './identity.js'
import { decodeMessage } from './api/utils.js'
import { StrategyFactory } from '@hoprnet/hopr-core/lib/channel-strategy.js'

// Metrics
const metric_processStartTime = create_gauge(
  'hoprd_gauge_startup_unix_time_seconds',
  'The unix timestamp at which the process was started'
)
const metric_nodeStartupTime = create_histogram_with_buckets(
  'hoprd_histogram_startup_time_seconds',
  'Time it takes for a node to start up',
  new Float64Array([5.0, 10.0, 30.0, 60.0, 120.0, 180.0, 300.0, 600.0, 1200.0])
)
const metric_timeToGreen = create_histogram_with_buckets(
  'hoprd_histogram_time_to_green_seconds',
  'Time it takes for a node to transition to the GREEN network state',
  new Float64Array([30.0, 60.0, 90.0, 120.0, 180.0, 240.0, 300.0, 420.0, 600.0, 900.0, 1200.0])
)
const metric_latency = create_histogram_with_buckets(
  'hoprd_histogram_message_latency_ms',
  'Histogram of measured received message latencies',
  new Float64Array([10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0, 20000.0])
)
const metric_version = create_multi_gauge('hoprd_mgauge_version', 'Executed version of HOPRd', ['version'])

// reading the version manually to ensure the path is read correctly
const packageFile = path.normalize(new URL('../package.json', import.meta.url).pathname)
const version = get_package_version(packageFile)
const on_avado = (process.env.AVADO ?? 'false').toLowerCase() === 'true'

function generateNodeOptions(argv: CliArgs, environment: ResolvedEnvironment): HoprOptions {
  let options: HoprOptions = {
    createDbIfNotExist: argv.init,
    announce: argv.announce,
    dataPath: argv.data,
    hosts: { ip4: argv.host },
    environment,
    allowLocalConnections: argv.allow_local_node_connections,
    allowPrivateConnections: argv.allow_private_node_connections,
    heartbeatInterval: argv.heartbeat_interval,
    heartbeatThreshold: argv.heartbeat_threshold,
    heartbeatVariance: argv.heartbeat_variance,
    networkQualityThreshold: argv.network_quality_threshold,
    onChainConfirmations: argv.on_chain_confirmations,
    testing: {
      announceLocalAddresses: argv.test_announce_local_addresses,
      preferLocalAddresses: argv.test_prefer_local_addresses,
      noWebRTCUpgrade: argv.test_no_webrtc_upgrade,
      noDirectConnections: argv.test_no_direct_connections
    }
  }

  if (argv.password !== undefined) {
    options.password = argv.password as string
  }

  if (isStrategy(argv.default_strategy)) {
    options.strategy = StrategyFactory.getStrategy(argv.default_strategy)
    if (argv.max_auto_channels !== undefined) {
      options.strategy.configure({
        max_channels: argv.max_auto_channels,
        auto_redeem_tickets: argv.auto_redeem_tickets
      })
    }
  }

  return options
}

// Parse the CLI arguments and return the processed object.
// This function may exit the calling process entirely if an error is
// encountered or the version or help are rendered.
export function parseCliArguments(args: string[]) {
  const mono_repo_path = new URL('../../../', import.meta.url).pathname
  let argv: CliArgs
  try {
    argv = parse_cli_arguments(args, process.env, mono_repo_path, process.env.HOME) as CliArgs
  } catch (err) {
    // both --version and --help are treated as errors, therefore we need some
    // special handling here to be able to return exit code 0 in such cases
    const message = err instanceof Error ? err.message : (err as String)
    if (message.startsWith('hoprd') || message.startsWith('HOPRd')) {
      console.log(err)
      process.exit(0)
    }
    console.error(err)
    process.exit(1)
  }
  return argv
}

async function addUnhandledPromiseRejectionHandler() {
  if (process.env.NODE_ENV !== 'production') {
    console.log(
      `Loading extended logger that enhances debugging of unhandled promise rejections. Disabled on production environments`
    )
    const { register: registerUnhandled, setLogger } = await import('trace-unhandled')

    registerUnhandled()
    setLogger((msg) => {
      console.error(msg)
    })
  }

  // Filter specific known promise rejection that cannot be handled for
  // one reason or the other
  setupPromiseRejectionFilter()
}

async function main() {
  // Starting with Node.js 15, undhandled promise rejections terminate the
  // process with a non-zero exit code, which makes debugging quite difficult.
  // Therefore adding a promise rejection handler to make sure that the origin of
  // the rejected promise can be detected.
  addUnhandledPromiseRejectionHandler()
  // Increase the default maximum number of event listeners
  ;(await import('events')).EventEmitter.defaultMaxListeners = 20

  metric_processStartTime.set(Date.now() / 1000)
  const metric_startupTimer = metric_nodeStartupTime.start_measure()

  let node: Hopr
  let logs = new LogStream()
  let state: State = {
    aliases: new Map(),
    settings: {
      includeRecipient: false,
      strategy: 'passive',
      autoRedeemTickets: false,
      maxAutoChannels: undefined
    }
  }

  const setState = (newState: State): void => {
    state = newState
  }

  const getState = (): State => {
    return state
  }

  let metric_timerToGreen = metric_timeToGreen.start_measure()

  const networkHealthChanged = (oldState: NetworkHealthIndicator, newState: NetworkHealthIndicator): void => {
    // Log the network health indicator state change (goes over the WS as well)
    logs.log(`Network health indicator changed: ${oldState} -> ${newState}`)
    logs.log(`NETWORK HEALTH: ${newState}`)
    if (metric_timerToGreen && newState == NetworkHealthIndicator.GREEN) {
      metric_timeToGreen.record_measure(metric_timerToGreen)
      metric_timerToGreen = undefined
    }
  }

  const logMessageToNode = (msg: Uint8Array): void => {
    logs.log(`#### NODE RECEIVED MESSAGE [${new Date().toISOString()}] ####`)
    try {
      let decodedMsg = decodeMessage(msg)
      logs.log(`Message: ${decodedMsg.msg}`)
      logs.log(`Latency: ${decodedMsg.latency} ms`)
      metric_latency.observe(decodedMsg.latency)

      // also send it tagged as message for apps to use
      logs.logMessage(decodedMsg.msg)
    } catch (err) {
      logs.log('Could not decode message', err instanceof Error ? err.message : 'Unknown error')
      logs.log(msg.toString())
    }
  }

  const argv = parseCliArguments(process.argv.slice(1))

  if (!argv.disable_api_authentication && argv.api) {
    if (argv.api_token == null) {
      throw Error(`Must provide --apiToken when --api is specified`)
    }
  }

  const environment = resolveEnvironment(argv.environment, argv.provider)
  let options = generateNodeOptions(argv, environment)
  if (argv.dry_run) {
    console.log(JSON.stringify(options, undefined, 2))
    process.exit(0)
  }

  try {
    logs.log(`This is HOPRd version ${version}`)
    metric_version.set([pickVersion(version)], 1.0)

    if (on_avado) {
      logs.log('This node appears to be running on an AVADO/Dappnode')
    }

    // 1. Find or create an identity
    const peerId = await getIdentity({
      initialize: argv.init,
      idPath: argv.identity,
      password: argv.password,
      useWeakCrypto: argv.test_use_weak_crypto,
      privateKey: argv.private_key
    })

    // 2. Create node instance
    logs.log('Creating HOPR Node')
    node = await createHoprNode(peerId, options, false)
    logs.logStatus('PENDING')

    // Subscribe to node events
    node.on('hopr:message', logMessageToNode)
    node.on('hopr:network-health-changed', networkHealthChanged)
    node.subscribeOnConnector('hopr:connector:created', () => {
      // 2.b - Connector has been created, and we can now trigger the next set of steps.
      logs.log('Connector has been loaded properly.')
      node.emit('hopr:monitoring:start')
    })
    node.once('hopr:monitoring:start', async () => {
      // 3. start all monitoring services, and continue with the rest of the setup.

      const startApiListen = setupAPI(
        node,
        logs,
        { getState, setState },
        {
          disableApiAuthentication: argv.disable_api_authentication,
          apiHost: argv.api_host,
          apiPort: argv.api_port,
          apiToken: argv.disable_api_authentication ? null : argv.api_token
        }
      )
      // start API server only if API flag is true
      if (argv.api) startApiListen()

      if (argv.health_check) {
        setupHealthcheck(node, logs, argv.health_check_host, argv.health_check_port)
      }

      logs.log(`Node address: ${node.getId().toString()}`)

      const ethAddr = node.getEthereumAddress().toHex()
      const fundsReq = new NativeBalance(SUGGESTED_NATIVE_BALANCE).toFormattedString()

      logs.log(`Node is not started, please fund this node ${ethAddr} with at least ${fundsReq}`)

      // 2.5 Await funding of wallet.
      await node.waitForFunds()
      logs.log('Node has been funded, starting...')

      // 3. Start the node.
      await node.start()

      // alias self
      state.aliases.set('me', node.getId())

      logs.logStatus('READY')
      logs.log('Node has started!')
      metric_nodeStartupTime.record_measure(metric_startupTimer)
    })

    // 2.a - Setup connector listener to bubble up to node. Emit connector creation.
    logs.log(`Ready to request on-chain connector to connect to provider.`)
    node.emitOnConnector('connector:create')
  } catch (e) {
    logs.log('Node failed to start:')
    logs.logFatalError('' + e)
    process.exit(1)
  }

  function stopGracefully(signal) {
    logs.log(`Process exiting with signal ${signal}`)
    process.exit()
  }

  process.on('uncaughtExceptionMonitor', (err, origin) => {
    // Make sure we get a log.
    logs.log(`FATAL ERROR, exiting with uncaught exception: ${origin} ${err}`)
  })

  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)
}

main()
