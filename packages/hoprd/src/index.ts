import path from 'path'

import {
  create_gauge,
  create_multi_gauge,
  get_package_version,
  Balance,
  BalanceType,
  setupPromiseRejectionFilter,
  SUGGESTED_NATIVE_BALANCE,
  create_histogram_with_buckets,
  pickVersion,
  defer,
  Address,
  debug,
  health_to_string
} from '@hoprnet/hopr-utils'
import {
  Health,
  createHoprNode,
  type Hopr,
  type HoprOptions,
  isStrategy,
  ResolvedNetwork,
  resolveNetwork
} from '@hoprnet/hopr-core'

import {
  parse_cli_arguments,
  fetch_configuration,
  parse_private_key,
  HoprdConfig,
  type Api,
  type CliArgs,
  hoprd_hoprd_initialize_crate
} from '../lib/hoprd_hoprd.js'
hoprd_hoprd_initialize_crate()

import {
  MessageInbox,
  HoprKeys,
  IdentityOptions,
  ApplicationData,
  MessageInboxConfiguration
} from '@hoprnet/hopr-utils'

import type { State } from './types.js'
import setupAPI from './api/index.js'
import setupHealthcheck from './healthcheck.js'

import { decodeMessage } from './api/utils.js'
import { type ChannelStrategyInterface, StrategyFactory } from '@hoprnet/hopr-core/lib/channel-strategy.js'
import { RPCH_MESSAGE_REGEXP } from './api/v3.js'

const log = debug('hoprd')

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
const on_dappnode = (process.env.DAPPNODE ?? 'false').toLowerCase() === 'true'

function generateNodeOptions(cfg: HoprdConfig, network: ResolvedNetwork): HoprOptions {
  let strategy: ChannelStrategyInterface

  if (isStrategy(cfg.strategy.name)) {
    strategy = StrategyFactory.getStrategy(cfg.strategy.name)
    strategy.configure({
      auto_redeem_tickets: cfg.strategy.auto_redeem_tickets,
      max_channels: cfg.strategy.max_auto_channels ?? undefined
    })
  } else {
    throw Error(`Invalid strategy selected`)
  }

  let options: HoprOptions = {
    createDbIfNotExist: cfg.db.initialize,
    announce: cfg.network_options.announce,
    dataPath: cfg.db.data,
    hosts: { ip4: cfg.host },
    network,
    allowLocalConnections: cfg.network_options.allow_local_node_connections,
    allowPrivateConnections: cfg.network_options.allow_private_node_connections,
    heartbeatInterval: cfg.heartbeat.interval,
    heartbeatThreshold: cfg.heartbeat.threshold,
    heartbeatVariance: cfg.heartbeat.variance,
    networkQualityThreshold: cfg.network_options.network_quality_threshold,
    onChainConfirmations: cfg.chain.on_chain_confirmations,
    checkUnrealizedBalance: cfg.chain.check_unrealized_balance,
    maxParallelConnections: cfg.network_options.max_parallel_connections,
    testing: {
      announceLocalAddresses: cfg.test.announce_local_addresses,
      preferLocalAddresses: cfg.test.prefer_local_addresses,
      noWebRTCUpgrade: cfg.test.no_webrtc_upgrade,
      noDirectConnections: cfg.test.no_direct_connections,
      localModeStun: cfg.test.local_mode_stun
    },
    password: cfg.identity.password,
    strategy,
    forceCreateDB: cfg.db.force_initialize,
    noRelay: cfg.network_options.no_relay,
    safeModule: {
      safeTransactionServiceProvider: cfg.safe_module.safe_transaction_service_provider,
      safeAddress: cfg.safe_module.safe_address,
      moduleAddress: cfg.safe_module.module_address
    }
  }

  if (isStrategy(cfg.strategy.name)) {
    options.strategy = StrategyFactory.getStrategy(cfg.strategy.name)
    options.strategy.configure({
      auto_redeem_tickets: cfg.strategy.auto_redeem_tickets,
      max_channels: cfg.strategy.max_auto_channels ?? undefined
    })
  }

  if (cfg.safe_module.safe_address) {
    options.safeModule.safeAddress = Address.deserialize(cfg.safe_module.safe_address.serialize())
  }
  if (cfg.safe_module.module_address) {
    options.safeModule.moduleAddress = Address.deserialize(cfg.safe_module.module_address.serialize())
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
  let inbox: MessageInbox
  let state: State = {
    aliases: new Map(),
    settings: {
      includeRecipient: false,
      strategy: 'passive',
      autoRedeemTickets: true,
      maxAutoChannels: undefined
    }
  }

  function stopGracefully(signal) {
    log(`Process exiting with signal ${signal}`)
    process.exit()
  }

  process.on('uncaughtExceptionMonitor', (err, origin) => {
    // Make sure we get a log.
    log(`FATAL ERROR, exiting with uncaught exception: ${origin} ${err}`)
  })

  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)

  const setState = (newState: State): void => {
    state = newState
  }

  const getState = (): State => {
    return state
  }

  let metric_timerToGreen = metric_timeToGreen.start_measure()

  const networkHealthChanged = (oldState: Health, newState: Health): void => {
    // Log the network health indicator state change (goes over the WS as well)
    log(`Network health indicator changed: ${health_to_string(oldState)} -> ${health_to_string(newState)}`)
    log(`NETWORK HEALTH: ${health_to_string(newState)}`)
    if (metric_timerToGreen && newState == Health.Green) {
      metric_timeToGreen.record_measure(metric_timerToGreen)
      metric_timerToGreen = undefined
    }
  }

  const logMessageToNode = async (data: ApplicationData) => {
    log(`#### NODE RECEIVED MESSAGE [${new Date().toISOString()}] ####`)
    try {
      let decodedMsg = decodeMessage(data.plain_text)
      log(`Message: ${decodedMsg.msg}`)
      log(`App tag: ${data.application_tag ?? 0}`)
      log(`Latency: ${decodedMsg.latency} ms`)
      metric_latency.observe(decodedMsg.latency)

      if (RPCH_MESSAGE_REGEXP.test(decodedMsg.msg)) {
        log(`RPCh: received message [${decodedMsg.msg}]`)
      }

      // Needs to be created new, because the incoming `data` is from serde_wasmbindgen and not a Rust WASM object
      let appData = new ApplicationData(data.application_tag, data.plain_text)
      await inbox.push(appData)
    } catch (err) {
      log('Could not decode message', err instanceof Error ? err.message : 'Unknown error', data.plain_text.toString())
    }
  }

  log('before parseCliArguments')
  const argv = parseCliArguments(process.argv.slice(1))
  log('after parseCliArguments')
  let cfg: HoprdConfig
  try {
    log('before fetch_configuration')
    cfg = fetch_configuration(argv as CliArgs) as HoprdConfig
    log('after fetch_configuration')
  } catch (err) {
    console.error(err)
    process.exit(1)
  }

  console.log('Node configuration: ' + cfg.as_redacted_string())

  if (argv.dry_run) {
    process.exit(0)
  }

  if (cfg.strategy.name) {
    state.settings.strategy = cfg.strategy.name
  }

  if (cfg.strategy.auto_redeem_tickets) {
    state.settings.autoRedeemTickets = cfg.strategy.auto_redeem_tickets
  }

  if (cfg.strategy.max_auto_channels) {
    state.settings.maxAutoChannels = cfg.strategy.max_auto_channels
  }

  const network = resolveNetwork(cfg.network, cfg.chain.provider)

  let options = generateNodeOptions(cfg, network)

  try {
    log(`This is HOPRd version ${version}`)
    metric_version.set([pickVersion(version)], 1.0)

    if (on_dappnode) {
      log('This node appears to be running on an Dappnode')
    }

    // 1. Find or create an identity
    const keypair = HoprKeys.init(
      new IdentityOptions(
        cfg.db.initialize,
        cfg.identity.file,
        cfg.identity.password,
        cfg.test.use_weak_crypto,
        cfg.identity.private_key === undefined ? undefined : parse_private_key(cfg.identity.private_key)
      )
    )

    log(`chain_key ${keypair.chain_key.public().to_hex(true)}`)
    log(`packet_key ${keypair.packet_key.public().to_peerid_str()}`)

    // 2. Create node instance
    log('Creating HOPR Node')
    node = await createHoprNode(keypair.chain_key, keypair.packet_key, options, false)
    log('Status: PENDING')

    // Subscribe to node events
    node.on('hopr:message', logMessageToNode)
    node.on('hopr:network-health-changed', networkHealthChanged)

    let continueStartup = defer<void>()
    node.subscribeOnConnector('hopr:connector:created', () => {
      // 2.b - Connector has been created, and we can now trigger the next set of steps.
      log('Connector has been loaded properly.')
      node.emit('hopr:monitoring:start')
      continueStartup.resolve()
    })

    // 2.a - Setup connector listener to bubble up to node. Emit connector creation.
    log(`Ready to request on-chain connector to connect to provider.`)
    node.emitOnConnector('connector:create')

    await continueStartup.promise

    // 3. start all monitoring services, and continue with the rest of the setup.

    let inboxCfg = new MessageInboxConfiguration()
    // TODO: pass configuration parameters for the inbox

    inbox = new MessageInbox(inboxCfg)

    let api = cfg.api as Api
    console.log(JSON.stringify(api, null, 2))
    const startApiListen = setupAPI(
      node,
      inbox,
      { getState, setState },
      {
        disableApiAuthentication: api.is_auth_disabled(),
        apiHost: api.host.ip,
        apiPort: api.host.port,
        apiToken: api.is_auth_disabled() ? null : api.auth_token()
      }
    )
    // start API server only if API flag is true
    if (cfg.api.enable) startApiListen()

    if (cfg.healthcheck.enable) {
      setupHealthcheck(node, cfg.healthcheck.host, cfg.healthcheck.port)
    }

    log(`Node address: ${node.getId().toString()}`)

    const ethAddr = node.getEthereumAddress().to_hex()
    const fundsReq = new Balance(SUGGESTED_NATIVE_BALANCE.toString(10), BalanceType.Native).to_formatted_string()

    log(`Node is not started, please fund this node ${ethAddr} with at least ${fundsReq}`)

    // 2.5 Await funding of wallet.
    await node.waitForFunds()
    log('Node has been funded, starting...')

    // 3. Start the node.
    await node.start()

    // alias self
    state.aliases.set('me', node.getId())

    log('Status: READY')
    log('Node has started!')
    metric_nodeStartupTime.record_measure(metric_startupTimer)

    // Won't return until node is terminated
    await node.startProcessing()
  } catch (e) {
    log('Node failed to start: ' + e)
    process.exit(1)
  }
}

main()
