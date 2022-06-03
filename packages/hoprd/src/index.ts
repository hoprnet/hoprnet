import { passwordStrength } from 'check-password-strength'
import { decode } from 'rlp'
import path from 'path'
import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'
import { setTimeout } from 'timers/promises'

import { NativeBalance, SUGGESTED_NATIVE_BALANCE } from '@hoprnet/hopr-utils'
import {
  default as Hopr,
  type HoprOptions,
  type NetworkHealthIndicator,
  createHoprNode,
  resolveEnvironment,
  supportedEnvironments,
  ResolvedEnvironment,
  HEARTBEAT_INTERVAL,
  HEARTBEAT_THRESHOLD,
  HEARTBEAT_INTERVAL_VARIANCE,
  NETWORK_QUALITY_THRESHOLD,
  CONFIRMATIONS
} from '@hoprnet/hopr-core'

import type { State } from './types.js'
import setupAPI from './api/index.js'
import setupHealthcheck from './healthcheck.js'
import { AdminServer } from './admin.js'
import { Commands } from './commands/index.js'
import { LogStream } from './logs.js'
import { getIdentity } from './identity.js'
import { register as registerUnhandled } from 'trace-unhandled'

import { setLogger } from 'trace-unhandled'

import * as wasm from '../lib/hoprd_misc.js'

const DEFAULT_ID_PATH = path.join(process.env.HOME, '.hopr-identity')

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

// Use environment-specific default data path
const defaultDataPath = path.join(
  path.dirname(new URL('../package.json', import.meta.url).pathname),
  defaultEnvironment()
)

const yargsInstance = yargs(hideBin(process.argv))

const argv = yargsInstance
  .env('HOPRD') // enable options to be set as environment variables with the HOPRD prefix
  .epilogue(
    'All CLI options can be configured through environment variables as well. CLI parameters have precedence over environment variables.'
  )
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
    describe:
      'Expose the Rest (V1, V2) and Websocket (V2) API on localhost:3001, requires --apiToken. "--rest" is deprecated [env: HOPRD_API]',
    default: false,
    alias: 'rest'
  })
  .option('apiHost', {
    string: true,
    describe:
      'Set host IP to which the Rest and Websocket API server will bind. "--restHost" is deprecated [env: HOPRD_API_HOST]',
    default: 'localhost',
    alias: 'restHost'
  })
  .option('apiPort', {
    number: true,
    describe:
      'Set host port to which the Rest and Websocket API server will bind. "--restPort" is deprecated [env: HOPRD_API_PORT]',
    default: 3001,
    alias: 'restPort'
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
  .option('apiToken', {
    string: true,
    describe: 'A REST API token and admin panel password for user authentication [env: HOPRD_API_TOKEN]',
    default: undefined
  })
  .option('privateKey', {
    string: true,
    describe: 'A private key to be used for the node [env: HOPRD_PRIVATE_KEY]',
    default: undefined
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
  .option('run', {
    string: true,
    describe: 'Run a single hopr command, same syntax as in hopr-admin [env: HOPRD_RUN]',
    default: ''
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
  .option('testAnnounceLocalAddresses', {
    boolean: true,
    describe: 'For testing local testnets. Announce local addresses [env: HOPRD_TEST_ANNOUNCE_LOCAL_ADDRESSES]',
    default: false
  })
  .option('testPreferLocalAddresses', {
    boolean: true,
    describe: 'For testing local testnets. Prefer local peers to remote [env: HOPRD_TEST_PREFER_LOCAL_ADDRESSES]',
    default: false
  })
  .option('testUseWeakCrypto', {
    boolean: true,
    describe: 'weaker crypto for faster node startup [env: HOPRD_TEST_USE_WEAK_CRYPTO]',
    default: false
  })
  .option('testNoAuthentication', {
    boolean: true,
    describe: 'no remote authentication for easier testing [env: HOPRD_TEST_NO_AUTHENTICATION]',
    default: false
  })
  .option('testNoDirectConnections', {
    boolean: true,
    describe:
      'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_DIRECT_CONNECTIONS]',
    default: false,
    hidden: true
  })
  .option('testNoWebRTCUpgrade', {
    boolean: true,
    describe:
      'NAT traversal testing: prevent nodes from establishing direct TCP connections [env: HOPRD_TEST_NO_WEB_RTC_UPGRADE]',
    default: false,
    hidden: true
  })
  .option('testNoUPNP', {
    boolean: true,
    describe:
      'NAT traversal testing: disable automatic detection of external IP address using UPNP [env: HOPRD_TEST_NO_UPNP]',
    default: false,
    hidden: true
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
      noDirectConnections: argv.testNoDirectConnections,
      noUPNP: argv.testNoUPNP
    }
  }

  if (argv.password !== undefined) {
    options.password = argv.password as string
  }

  return options
}

function addUnhandledPromiseRejectionHandler() {
  registerUnhandled()
  setLogger((msg) => {
    console.error(msg)
  })

  // See https://github.com/hoprnet/hoprnet/issues/3755
  process.on('unhandledRejection', (reason: any, _promise: Promise<any>) => {
    if (reason.message && reason.message.toString) {
      const msgString = reason.toString()

      // Only silence very specific errors
      if (
        msgString.match(/read ECONNRESET/) ||
        msgString.match(/write ECONNRESET/) ||
        msgString.match(/The operation was aborted/)
      ) {
        console.error('Unhandled promise rejection silenced')
        return
      }
    }

    console.warn('UnhandledPromiseRejectionWarning')
    console.log(reason)
    process.exit(1)
  })
}

async function main() {
  // Starting with Node.js 15, undhandled promise rejections terminate the
  // process with a non-zero exit code, which makes debugging quite difficult.
  // Therefore adding a promise rejection handler to make sure that the origin of
  // the rejected promise can be detected.
  addUnhandledPromiseRejectionHandler()
  // Increase the default maximum number of event listeners
  ;(await import('events')).EventEmitter.defaultMaxListeners = 20

  let node: Hopr
  let logs = new LogStream()
  let adminServer: AdminServer = undefined
  let cmds: Commands
  // As the daemon aims to maintain for the time being
  // both APIv1 and APIv2 (hopr-admin / myne-chat), we need
  // to ensure that daemon's state can be used by both APIs.
  let state: State = {
    aliases: new Map(),
    settings: {
      includeRecipient: false,
      strategy: 'passive'
    }
  }
  function setState(newState: State): void {
    state = newState
  }
  function getState(): State {
    return state
  }

  function networkHealthChanged(oldState: NetworkHealthIndicator, newState: NetworkHealthIndicator) {
    // Log the network health indicator state change (goes over the WS as well)
    logs.log(`Network health indicator changed: ${oldState} -> ${newState}`)
    logs.log(`NETWORK HEALTH: ${newState}`)
  }

  function logMessageToNode(msg: Uint8Array) {
    logs.log(`#### NODE RECEIVED MESSAGE [${new Date().toISOString()}] ####`)
    try {
      let [decoded, time] = decode(msg) as [Buffer, Buffer]
      logs.log(`Message: ${decoded.toString()}`)
      logs.log(`Latency: ${Date.now() - parseInt(time.toString('hex'), 16)}ms`)

      // also send it tagged as message for apps to use
      logs.logMessage(decoded.toString())
    } catch (err) {
      logs.log('Could not decode message', err)
      logs.log(msg.toString())
    }
  }

  if (!argv.testNoAuthentication && (argv.api || argv.admin)) {
    if (argv.apiToken == null) {
      throw Error(`Must provide --apiToken when --api, --rest or --admin is specified`)
    }
    const { contains: hasSymbolTypes, length }: { contains: string[]; length: number } = passwordStrength(argv.apiToken)
    for (const requiredSymbolType of ['uppercase', 'lowercase', 'symbol', 'number']) {
      if (!hasSymbolTypes.includes(requiredSymbolType)) {
        throw new Error(`API token must include a ${requiredSymbolType}`)
      }
    }
    if (length < 8) {
      throw new Error(`API token must be at least 8 characters long`)
    }
  }

  const apiToken = argv.testNoAuthentication ? null : argv.apiToken

  // We need to setup the admin server before the HOPR node
  // as if the HOPR node fails, we need to put an error message up.
  if (argv.admin) {
    adminServer = new AdminServer(logs, argv.adminHost, argv.adminPort)
    await adminServer.setup()
  }

  const environment = resolveEnvironment(argv.environment, argv.provider)
  let options = generateNodeOptions(environment)
  if (argv.dryRun) {
    console.log(JSON.stringify(options, undefined, 2))
    process.exit(0)
  }

  try {
    let packageFile = path.normalize(new URL('../package.json', import.meta.url).pathname)
    logs.log(`This is hoprd version ${wasm.get_package_version(packageFile)}`)

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
    node.on('hopr:message', logMessageToNode)
    node.on('hopr:network-health-changed', networkHealthChanged)
    node.subscribeOnConnector('hopr:connector:created', () => {
      // 2.b - Connector has been created, and we can now trigger the next set of steps.
      logs.log('Connector has been loaded properly.')
      node.emit('hopr:monitoring:start')
    })
    node.once('hopr:monitoring:start', async () => {
      // 3. start all monitoring services, and continue with the rest of the setup.

      if (argv.api || argv.admin) {
        /*
          When `--api` is used, we turn on Rest API v1, v2 and WS API v2.
          When `--admin` is used, we turn on WS API v1 only.
        */
        setupAPI(
          node,
          logs,
          { getState, setState },
          {
            ...argv,
            apiToken
          },
          adminServer // api V1: required by hopr-admin
        )
      }

      if (argv.healthCheck) {
        setupHealthcheck(node, logs, argv.healthCheckHost, argv.healthCheckPort)
      }

      logs.log(`Node address: ${node.getId().toB58String()}`)

      const ethAddr = node.getEthereumAddress().toHex()
      const fundsReq = new NativeBalance(SUGGESTED_NATIVE_BALANCE).toFormattedString()

      logs.log(`Node is not started, please fund this node ${ethAddr} with at least ${fundsReq}`)

      // 2.5 Await funding of wallet.
      await node.waitForFunds()
      logs.log('Node has been funded, starting...')

      // 3. Start the node.
      await node.start()
      cmds = new Commands(node, { setState, getState })

      if (adminServer) {
        adminServer.registerNode(node, cmds)
      }

      logs.logStatus('READY')
      logs.log('Node has started!')

      if (argv.run && argv.run !== '') {
        // Run a single command and then exit.
        // We support multiple semicolon separated commands
        let toRun = argv.run.split(';').map((c: string) =>
          // Remove obsolete ' and "
          c.replace(/"/g, '')
        )

        for (let c of toRun) {
          console.error('$', c)
          if (c === 'daemonize') {
            return
          }

          await cmds.execute((msg) => {
            logs.log(msg)
          }, c)
        }
        // Wait for actions to take place
        await setTimeout(1e3)
        await node.stop()
        return
      }
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
