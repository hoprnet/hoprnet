#!/usr/bin/env node

import { passwordStrength } from 'check-password-strength'
import { decode } from 'rlp'
import path from 'path'
import yargs from 'yargs/yargs'
import { terminalWidth } from 'yargs'
import { setTimeout } from 'timers/promises'

import Hopr, { createHoprNode } from '@hoprnet/hopr-core'
import { NativeBalance, SUGGESTED_NATIVE_BALANCE } from '@hoprnet/hopr-utils'
import { resolveEnvironment, supportedEnvironments, ResolvedEnvironment } from '@hoprnet/hopr-core'

import setupAPI from './api'
import setupHealthcheck from './healthcheck'
import { AdminServer } from './admin'
import { Commands } from './commands'
import { LogStream } from './logs'
import { getIdentity } from './identity'

import type { HoprOptions } from '@hoprnet/hopr-core'
import { setLogger } from 'trace-unhandled'

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

const argv = yargs(process.argv.slice(2))
  .option('environment', {
    string: true,
    describe: 'Environment id which the node shall run on',
    choices: supportedEnvironments().map((env) => env.id),
    default: defaultEnvironment()
  })
  .option('host', {
    string: true,
    describe: 'The network host to run the HOPR node on.',
    default: '0.0.0.0:9091'
  })
  .option('announce', {
    boolean: true,
    describe: 'Announce public IP to the network',
    default: false
  })
  .option('admin', {
    boolean: true,
    describe: 'Run an admin interface on localhost:3000, requires --apiToken',
    default: false
  })
  .option('rest', {
    boolean: true,
    describe: 'Expose the Rest API on localhost:3001, requires --apiToken',
    default: false
  })
  .option('restHost', {
    string: true,
    describe: 'Set host IP to which the Rest API server will bind',
    default: 'localhost'
  })
  .option('restPort', {
    number: true,
    describe: 'Set host port to which the Rest API server will bind',
    default: 3001
  })
  .option('healthCheck', {
    boolean: true,
    describe: 'Run a health check end point on localhost:8080',
    default: false
  })
  .option('healthCheckHost', {
    string: true,
    describe: 'Updates the host for the healthcheck server',
    default: 'localhost'
  })
  .option('healthCheckPort', {
    number: true,
    describe: 'Updates the port for the healthcheck server',
    default: 8080
  })
  .option('forwardLogs', {
    boolean: true,
    describe: 'Forwards all your node logs to a public available sink',
    default: false
  })
  .option('forwardLogsProvider', {
    string: true,
    describe: 'A provider url for the logging sink node to use',
    default: 'https://ceramic-clay.3boxlabs.com'
  })
  .option('password', {
    string: true,
    describe: 'A password to encrypt your keys',
    default: ''
  })
  .option('apiToken', {
    string: true,
    describe: 'A REST API token and admin panel password for user authentication',
    default: undefined
  })
  .option('privateKey', {
    string: true,
    describe: 'A private key to be used for your HOPR node',
    default: undefined
  })
  .option('identity', {
    string: true,
    describe: 'The path to the identity file',
    default: DEFAULT_ID_PATH
  })
  .option('run', {
    string: true,
    describe: 'Run a single hopr command, same syntax as in hopr-admin',
    default: ''
  })
  .option('dryRun', {
    boolean: true,
    describe: 'List all the options used to run the HOPR node, but quit instead of starting',
    default: false
  })
  .option('data', {
    string: true,
    describe: 'manually specify the database directory to use',
    default: ''
  })
  .option('init', {
    boolean: true,
    describe: "initialize a database if it doesn't already exist",
    default: false
  })
  .option('adminHost', {
    string: true,
    describe: 'Host to listen to for admin console',
    default: 'localhost'
  })
  .option('adminPort', {
    string: true,
    describe: 'Port to listen to for admin console',
    default: 3000
  })
  .option('testAnnounceLocalAddresses', {
    boolean: true,
    describe: 'For testing local testnets. Announce local addresses.',
    default: false
  })
  .option('testPreferLocalAddresses', {
    boolean: true,
    describe: 'For testing local testnets. Prefer local peers to remote.',
    default: false
  })
  .option('testUseWeakCrypto', {
    boolean: true,
    describe: 'weaker crypto for faster node startup',
    default: false
  })
  .option('testNoAuthentication', {
    boolean: true,
    describe: 'no remote authentication for easier testing',
    default: false
  })
  .option('testNoDirectConnections', {
    boolean: true,
    describe: 'NAT traversal testing: prevent nodes from establishing direct TCP connections',
    default: false,
    hidden: true
  })
  .option('testNoWebRTCUpgrade', {
    boolean: true,
    describe: 'NAT traversal testing: prevent nodes from establishing direct TCP connections',
    default: false,
    hidden: true
  })
  .option('testNoUPNP', {
    boolean: true,
    describe: 'NAT traversal testing: disable automatic detection of external IP address using UPNP',
    default: false,
    hidden: true
  })
  .wrap(Math.min(120, terminalWidth()))
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
    hosts: parseHosts(),
    environment,
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

  if (argv.data && argv.data !== '') {
    options.dbPath = argv.data
  }
  return options
}

function addUnhandledPromiseRejectionHandler() {
  require('trace-unhandled/register')
  setLogger((msg) => {
    console.error(msg)
    process.exit(1)
  })
}

async function main() {
  // Starting with Node.js 15, undhandled promise rejections terminate the
  // process with a non-zero exit code, which makes debugging quite difficult.
  // Therefore adding a promise rejection handler to make sure that the origin of
  // the rejected promise can be detected.
  addUnhandledPromiseRejectionHandler()

  let node: Hopr
  let logs = new LogStream(argv.forwardLogs)
  let adminServer = undefined
  let cmds: Commands

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

  if (logs.isReadyForPublicLogging()) {
    const publicLogsId = await logs.enablePublicLoggingNode(argv.forwardLogsProvider)
    logs.log(`Your unique Log Id is ${publicLogsId}`)
    logs.log(`View logs at https://documint.net/${publicLogsId}`)
    logs.startLoggingQueue()
  }

  if (!argv.testNoAuthentication && (argv.rest || argv.admin)) {
    if (argv.apiToken == null) {
      throw Error(`Must provide --apiToken when --admin or --rest is specified`)
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

  if (argv.admin) {
    // We need to setup the admin server before the HOPR node
    // as if the HOPR node fails, we need to put an error message up.
    let apiToken = argv.apiToken
    if (argv.testNoAuthentication) {
      apiToken = null
    }
    adminServer = new AdminServer(logs, argv.adminHost, argv.adminPort, apiToken)
    await adminServer.setup()
  }

  const environment = resolveEnvironment(argv.environment)
  let options = generateNodeOptions(environment)
  if (argv.dryRun) {
    console.log(JSON.stringify(options, undefined, 2))
    process.exit(0)
  }

  try {
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
    node.on('hopr:connector:created', () => {
      // 2.b - Connector has been created, and we can now trigger the next set of steps.
      logs.log('Connector has been loaded properly.')
      node.emit('hopr:monitoring:start')
    })
    node.on('hopr:monitoring:start', async () => {
      // 3. start all monitoring services, and continue with the rest of the setup.

      if (argv.rest) {
        setupAPI(node, logs, argv)
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
      cmds = new Commands(node)

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
          console.log(`command: `, c)
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
    node.subscribeOnConnector('connector:created', () => node.emit('hopr:connector:created'))
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
