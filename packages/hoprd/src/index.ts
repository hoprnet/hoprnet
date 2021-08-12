#!/usr/bin/env node
import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import { NativeBalance, SUGGESTED_NATIVE_BALANCE } from '@hoprnet/hopr-utils'
import { decode } from 'rlp'
import { Commands } from './commands'
import { LogStream } from './logs'
import { AdminServer } from './admin'
import yargs from 'yargs/yargs'
import { terminalWidth } from 'yargs'
import setupAPI from './api'
import { getIdentity } from './identity'
import path from 'path'
import { passwordStrength } from 'check-password-strength'
import type { ProtocolConfig } from '@hoprnet/hopr-core'

const DEFAULT_ID_PATH = path.join(process.env.HOME, '.hopr-identity')

const pkg = require('../../../package.json')

const argv = yargs(process.argv.slice(2))
  .option('provider', {
    describe: 'A provider url for the Network you specified',
    default: 'https://still-patient-forest.xdai.quiknode.pro/f0cdbd6455c0b3aea8512fc9e7d161c1c0abf66a/'
  })
  .option('environment', {
    describe: 'Environment id, one of the ids defined in protocol-config.json',
    default: pkg.hopr.environment_id
  })
  .option('host', {
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
    describe: 'Run a rest interface on localhost:3001, requires --apiToken',
    default: false
  })
  .option('restHost', {
    describe: 'Updates the host for the rest server',
    default: 'localhost'
  })
  .option('restPort', {
    describe: 'Updates the port for the rest server',
    default: 3001
  })
  .option('healthCheck', {
    boolean: true,
    describe: 'Run a health check end point on localhost:8080',
    default: false
  })
  .option('healthCheckHost', {
    describe: 'Updates the host for the healthcheck server',
    default: 'localhost'
  })
  .option('healthCheckPort', {
    describe: 'Updates the port for the healthcheck server',
    default: 8080
  })
  .option('forwardLogs', {
    boolean: true,
    describe: 'Forwards all your node logs to a public available sink',
    default: false
  })
  .option('forwardLogsProvider', {
    describe: 'A provider url for the logging sink node to use',
    default: 'https://ceramic-clay.3boxlabs.com'
  })
  .option('password', {
    describe: 'A password to encrypt your keys',
    default: ''
  })
  .option('apiToken', {
    describe: 'A REST API token and admin panel password for user authentication',
    string: true,
    default: undefined
  })
  .option('privateKey', {
    describe: 'A private key to be used for your HOPR node',
    string: true,
    default: undefined
  })
  .option('identity', {
    describe: 'The path to the identity file',
    default: DEFAULT_ID_PATH
  })
  .option('run', {
    describe: 'Run a single hopr command, same syntax as in hopr-admin',
    default: ''
  })
  .option('dryRun', {
    boolean: true,
    describe: 'List all the options used to run the HOPR node, but quit instead of starting',
    default: false
  })
  .option('data', {
    describe: 'manually specify the database directory to use',
    default: ''
  })
  .option('init', {
    boolean: true,
    describe: "initialize a database if it doesn't already exist",
    default: false
  })
  .option('adminHost', {
    describe: 'Host to listen to for admin console',
    default: 'localhost'
  })
  .option('adminPort', {
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

async function generateNodeOptions(): Promise<HoprOptions> {
  let options: HoprOptions = {
    createDbIfNotExist: argv.init,
    announce: argv.announce,
    hosts: parseHosts(),
    announceLocalAddresses: argv.testAnnounceLocalAddresses,
    preferLocalAddresses: argv.testPreferLocalAddresses,
    environment: undefined
  }

  if (argv.password !== undefined) {
    options.password = argv.password as string
  }

  if (argv.data && argv.data !== '') {
    options.dbPath = argv.data
  }

  const protocolConfig = require('../protocol-config.json') as ProtocolConfig
  for (const environment of protocolConfig.environments) {
    if (environment.id === argv.environment) {
      for (const network of protocolConfig.networks) {
        if (network.id === environment.network_id) {
          options.environment = {
            id: environment.id,
            network,
            channel_contract_deploy_block: environment.channel_contract_deploy_block,
            token_contract_address: environment.token_contract_address,
            channels_contract_address: environment.channels_contract_address
          }
        }
      }

    }
  }

  if (!options.environment) {
    throw new Error(`failed to find environment with id ${argv.environment} in protocol-config.json`)
  }

  return options
}

function addUnhandledPromiseRejectionHandler() {
  process.on('unhandledRejection', (reason: any, promise: Promise<any>) => {
    console.error('Unhandled Rejection at:', promise, 'reason:', reason)
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
      logs.log('Message:', decoded.toString())
      logs.log('Latency:', Date.now() - parseInt(time.toString('hex'), 16) + 'ms')
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

  logs.log('Creating HOPR Node')
  let options = await generateNodeOptions()
  if (argv.dryRun) {
    console.log(JSON.stringify(options, undefined, 2))
    process.exit(0)
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
  try {
    node = new Hopr(peerId, options)
    logs.logStatus('PENDING')
    logs.log('Creating HOPR Node')
    node.on('hopr:message', logMessageToNode)

    // 2.1 start all monitoring services

    if (argv.rest) {
      setupAPI(node, logs, argv)
    }

    if (argv.healthCheck) {
      const http = require('http')
      const service = require('restana')()
      service.get('/healthcheck/v1/version', (_, res) => res.send(node.getVersion()))
      const hostname = argv.healthCheckHost
      const port = argv.healthCheckPort
      const server = http.createServer(service).on('error', (err) => {
        throw err
      })
      server.listen(port, hostname, (err) => {
        if (err) throw err
        logs.log(`Healthcheck server on ${hostname} listening on port ${port}`)
      })
    }

    const ethAddr = (await node.getEthereumAddress()).toHex()
    const fundsReq = new NativeBalance(SUGGESTED_NATIVE_BALANCE).toFormattedString()

    logs.log(`Node is not started, please fund this node ${ethAddr} with atleast ${fundsReq}`)
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
      let toRun = argv.run.split(';')

      for (let c of toRun) {
        console.error('$', c)
        if (c === 'daemonize') {
          return
        }
        await cmds.execute((msg) => {
          logs.log(msg)
          console.log(msg)
        }, c)
      }
      await node.stop()
      process.exit(0)
    }
  } catch (e) {
    console.log(e)
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

  process.once('exit', stopGracefully)
  process.on('SIGINT', stopGracefully)
  process.on('SIGTERM', stopGracefully)
}

main()
