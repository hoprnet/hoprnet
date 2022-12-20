import path from 'path'
import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'

import {
  create_gauge,
  create_multi_gauge,
  get_package_version,
  loadJson,
  NativeBalance,
  setupPromiseRejectionFilter,
  SUGGESTED_NATIVE_BALANCE,
  create_histogram_with_buckets,
  pickVersion
} from '@hoprnet/hopr-utils'
import {
  CONFIRMATIONS,
  createHoprNode,
  default as Hopr,
  HEARTBEAT_INTERVAL,
  HEARTBEAT_INTERVAL_VARIANCE,
  HEARTBEAT_THRESHOLD,
  type HoprOptions,
  NETWORK_QUALITY_THRESHOLD,
  NetworkHealthIndicator,
  ResolvedEnvironment,
  resolveEnvironment,
  supportedEnvironments
} from '@hoprnet/hopr-core'

import type { State } from './types.js'
import setupAPI from './api/index.js'
import setupHealthcheck from './healthcheck.js'
import { AdminServer } from './admin.js'
import { LogStream } from './logs.js'
import { getIdentity } from './identity.js'
import { decodeMessage } from './api/utils.js'

const DEFAULT_ID_PATH = path.join(process.env.HOME, '.hopr-identity')

export type DefaultEnvironment = {
  id?: string
}

function defaultEnvironment(): string {
  try {
    const config = loadJson('../default-environment.json') as DefaultEnvironment
    return config?.id || ''
  } catch (error) {
    // its ok if the file isn't there or cannot be read
    return ''
  }
}

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

// Use environment-specific default data path
const defaultDataPath = path.join(process.cwd(), 'hoprd-db', defaultEnvironment())

// reading the version manually to ensure the path is read correctly
const packageFile = path.normalize(new URL('../package.json', import.meta.url).pathname)
const version = get_package_version(packageFile)
const on_avado = (process.env.AVADO ?? 'false').toLowerCase() === 'true'

const yargsInstance = yargs(hideBin(process.argv))

const argv = yargsInstance
  .env('HOPRD') // enable options to be set as environment variables with the HOPRD prefix
  .epilogue(
    'All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.'
  )
  .version(version)
  .option('environment', {
    string: true,
    describe: 'Environment id which the node shall run on (HOPRD_ENVIRONMENT)',
    choices: supportedEnvironments().map((env) => env.id),
    default: defaultEnvironment()
  })
  .option('host', {
    string: true,
    describe: 'The network host to run the HOPR node on [env: HOPRD_HOST]',
    default: '0.0.0.0:9091'
  })
  .option('announce', {
    boolean: true,
    describe: 'Announce public IP to the network [env: HOPRD_ANNOUNCE]',
    default: false
  })
  .option('admin', {
    boolean: true,
    describe: 'Run an admin interface on localhost:3000, requires --apiToken [env: HOPRD_ADMIN]',
    default: false
  })
  .option('adminHost', {
    string: true,
    describe: 'Host to listen to for admin console [env: HOPRD_ADMIN_HOST]',
    default: 'localhost'
  })
  .option('adminPort', {
    string: true,
    describe: 'Port to listen to for admin console [env: HOPRD_ADMIN_PORT]',
    default: 3000
  })
  .option('api', {
    boolean: true,
    describe: 'Expose the API on localhost:3001. [env: HOPRD_API]',
    default: false
  })
  .option('apiHost', {
    string: true,
    describe: 'Set host IP to which the API server will bind. [env: HOPRD_API_HOST]',
    default: 'localhost'
  })
  .option('apiPort', {
    number: true,
    describe: 'Set host port to which the API server will bind. [env: HOPRD_API_PORT]',
    default: 3001
  })
  .option('apiToken', {
    string: true,
    describe: 'A REST API token and admin panel password for user authentication [env: HOPRD_API_TOKEN]',
    default: undefined,
    conflicts: 'disableApiAuthentication'
  })
  .option('healthCheck', {
    boolean: true,
    describe: 'Run a health check end point on localhost:8080 [env: HOPRD_HEALTH_CHECK]',
    default: false
  })
  .option('healthCheckHost', {
    string: true,
    describe: 'Updates the host for the healthcheck server [env: HOPRD_HEALTH_CHECK_HOST]',
    default: 'localhost'
  })
  .option('healthCheckPort', {
    number: true,
    describe: 'Updates the port for the healthcheck server [env: HOPRD_HEALTH_CHECK_PORT]',
    default: 8080
  })
  .option('password', {
    string: true,
    describe: 'A password to encrypt your keys [env: HOPRD_PASSWORD]',
    default: ''
  })
  .option('provider', {
    string: true,
    describe: 'A custom RPC provider to be used for the node to connect to blockchain [env: HOPRD_PROVIDER]'
  })
  .option('identity', {
    string: true,
    describe: 'The path to the identity file [env: HOPRD_IDENTITY]',
    default: DEFAULT_ID_PATH
  })
  .option('dryRun', {
    boolean: true,
    describe: 'List all the options used to run the HOPR node, but quit instead of starting [env: HOPRD_DRY_RUN]',
    default: false
  })
  .option('data', {
    string: true,
    describe: 'manually specify the data directory to use [env: HOPRD_DATA]',
    default: defaultDataPath
  })
  .option('init', {
    boolean: true,
    describe: "initialize a database if it doesn't already exist [env: HOPRD_INIT]",
    default: false
  })
  .option('privateKey', {
    hidden: true,
    string: true,
    describe: 'A private key to be used for the node [env: HOPRD_PRIVATE_KEY]',
    default: undefined
  })
  .option('allowLocalNodeConnections', {
    boolean: true,
    describe: 'Allow connections to other nodes running on localhost [env: HOPRD_ALLOW_LOCAL_NODE_CONNECTIONS]',
    default: false
  })
  .option('allowPrivateNodeConnections', {
    boolean: true,
    describe:
      'Allow connections to other nodes running on private addresses [env: HOPRD_ALLOW_PRIVATE_NODE_CONNECTIONS]',
    default: false
  })
  .option('disableApiAuthentication', {
    boolean: true,
    describe: 'Disable authentication for the API endpoints [env: HOPRD_DISABLE_API_AUTHENTICATION]',
    default: false
  })
  .option('testAnnounceLocalAddresses', {
    hidden: true,
    boolean: true,
    describe: 'For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES]',
    default: false
  })
  .option('testPreferLocalAddresses', {
    hidden: true,
    boolean: true,
    describe: 'For testing local testnets. Prefer local peers to remote [env: HOPRD_TEST_PREFER_LOCAL_ADDRESSES]',
    default: false
  })
  .option('testUseWeakCrypto', {
    hidden: true,
    boolean: true,
    describe: 'weaker crypto for faster node startup [env: HOPRD_TEST_USE_WEAK_CRYPTO]',
    default: false
  })
  .option('testNoDirectConnections', {
    hidden: true,
    boolean: true,
    describe:
      'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_DIRECT_CONNECTIONS]',
    default: false
  })
  .option('testNoWebRTCUpgrade', {
    hidden: true,
    boolean: true,
    describe:
      'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_WEB_RTC_UPGRADE]',
    default: false
  })
  .option('heartbeatInterval', {
    number: true,
    describe:
      'Interval in milliseconds in which the availability of other nodes get measured [env: HOPRD_HEARTBEAT_INTERVAL]',
    default: HEARTBEAT_INTERVAL
  })
  .option('heartbeatThreshold', {
    number: true,
    describe:
      "Timeframe in milliseconds after which a heartbeat to another peer is performed, if it hasn't been seen since [env: HOPRD_HEARTBEAT_THRESHOLD]",
    default: HEARTBEAT_THRESHOLD
  })
  .option('heartbeatVariance', {
    number: true,
    describe: 'Upper bound for variance applied to heartbeat interval in milliseconds [env: HOPRD_HEARTBEAT_VARIANCE]',
    default: HEARTBEAT_INTERVAL_VARIANCE
  })
  .option('networkQualityThreshold', {
    number: true,
    describe: 'Miniumum quality of a peer connection to be considered usable [env: HOPRD_NETWORK_QUALITY_THRESHOLD]',
    default: NETWORK_QUALITY_THRESHOLD
  })
  .option('onChainConfirmations', {
    number: true,
    describe: 'Number of confirmations required for on-chain transactions [env: HOPRD_ON_CHAIN_CONFIRMATIONS]',
    default: CONFIRMATIONS
  })
  .showHidden('show-hidden', 'show all options, including debug options')
  .strict()
  .wrap(Math.min(120, yargsInstance.terminalWidth()))
  .parseSync()

function parseHosts(): HoprOptions['hosts'] {
  const hosts: HoprOptions['hosts'] = {}
  if (argv.host !== undefined) {
    const str = argv.host.replace(/\/\/.+/, '').trim()
    const params = str.match(/([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\:([0-9]{1,6})/)
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv4 host. Got ${str}`)
    }

    hosts.ip4 = {
      ip: params[1],
      port: parseInt(params[2])
    }
  }
  return hosts
}

function generateNodeOptions(environment: ResolvedEnvironment): HoprOptions {
  let options: HoprOptions = {
    createDbIfNotExist: argv.init,
    announce: argv.announce,
    dataPath: argv.data,
    hosts: parseHosts(),
    environment,
    allowLocalConnections: argv.allowLocalNodeConnections,
    allowPrivateConnections: argv.allowPrivateNodeConnections,
    heartbeatInterval: argv.heartbeatInterval,
    heartbeatThreshold: argv.heartbeatThreshold,
    heartbeatVariance: argv.heartbeatVariance,
    networkQualityThreshold: argv.networkQualityThreshold,
    onChainConfirmations: argv.onChainConfirmations,
    testing: {
      announceLocalAddresses: argv.testAnnounceLocalAddresses,
      preferLocalAddresses: argv.testPreferLocalAddresses,
      noWebRTCUpgrade: argv.testNoWebRTCUpgrade,
      noDirectConnections: argv.testNoDirectConnections
    }
  }

  if (argv.password !== undefined) {
    options.password = argv.password as string
  }

  return options
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
  let adminServer: AdminServer = undefined
  let state: State = {
    aliases: new Map(),
    settings: {
      includeRecipient: false,
      strategy: 'passive'
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

  if (!argv.disableApiAuthentication && argv.api) {
    if (argv.apiToken == null) {
      throw Error(`Must provide --apiToken when --api is specified`)
    }
    if (argv.apiToken.length < 8) {
      throw new Error(`API token must be at least 8 characters long`)
    }
  }

  const apiToken = argv.disableApiAuthentication ? null : argv.apiToken

  // We need to setup the admin server before the HOPR node
  // as if the HOPR node fails, we need to put an error message up.
  if (argv.admin) {
    adminServer = new AdminServer(logs, argv.adminHost, argv.adminPort)
    try {
      await adminServer.setup()
    } catch (err) {
      console.error(err)
      process.exit(1)
    }
  }

  const environment = resolveEnvironment(argv.environment, argv.provider)
  let options = generateNodeOptions(environment)
  if (argv.dryRun) {
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
      useWeakCrypto: argv.testUseWeakCrypto,
      privateKey: argv.privateKey
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
          ...argv,
          apiHost: argv.apiHost,
          apiPort: argv.apiPort,
          apiToken
        }
      )
      // start API server only if API flag is true
      if (argv.api) startApiListen()

      if (argv.healthCheck) {
        setupHealthcheck(node, logs, argv.healthCheckHost, argv.healthCheckPort)
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

      if (adminServer) {
        adminServer.registerNode(node)
      }

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
    if (!argv.admin) {
      // If the admin interface is running, we should keep process alive
      process.exit(1)
    }
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
