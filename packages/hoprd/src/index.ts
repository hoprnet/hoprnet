#!/usr/bin/env node
import Hopr, { FULL_VERSION } from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { decode } from 'rlp'
// @ts-ignore
import Multihash from 'multihashes'
import { getBootstrapAddresses } from '@hoprnet/hopr-utils'
import { Commands } from './commands'
import { LogStream } from './logs'
import { AdminServer } from './admin'
import * as yargs from 'yargs'

const argv = yargs
  .option('network', {
    describe: 'Which network to run the HOPR node on',
    default: 'ETHEREUM',
    choices: ['ETHEREUM']
  })
  .option('provider', {
    describe: 'A provider url for the Network you specified',
    default: 'wss://bsc-ws-node.nariox.org:443'
  })
  .option('host', {
    describe: 'The network host to run the HOPR node on.',
    default: '0.0.0.0:9091'
  })
  .option('admin', {
    boolean: true,
    describe: 'Run an admin interface on localhost:3000',
    default: false
  })
  .option('rest', {
    boolean: true,
    describe: 'Run a rest interface on localhost:3001',
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
  .option('password', {
    describe: 'A password to encrypt your keys'
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
  .option('runAsBootstrap', {
    boolean: true,
    describe: 'run as a bootstrap node',
    default: false
  })
  .option('bootstrapServers', {
    describe: 'manually specify bootstrap servers',
    default: undefined
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
  .option('settings', {
    descripe: 'Settings, same as in the repl (JSON)',
    default: '{}'
  })
  .wrap(Math.min(120, yargs.terminalWidth())).argv

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
    debug: Boolean(process.env.HOPR_DEBUG),
    bootstrapNode: argv.runAsBootstrap,
    createDbIfNotExist: argv.init,
    network: argv.network,
    bootstrapServers: argv.runAsBootstrap ? [] : [...(await getBootstrapAddresses(argv.bootstrapServers)).values()],
    provider: argv.provider,
    hosts: parseHosts()
  }

  if (argv.password !== undefined) {
    options.password = argv.password as string
  }

  if (argv.data && argv.data !== '') {
    options.dbPath = argv.data
  }
  return options
}

async function main() {
  let node: Hopr<HoprCoreConnector>
  let logs = new LogStream()
  let adminServer = undefined
  let settings: any = {}

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

  if (argv.settings) {
    settings = JSON.parse(argv.settings)
  }

  if (argv.admin) {
    // We need to setup the admin server before the HOPR node
    // as if the HOPR node fails, we need to put an error message up.
    adminServer = new AdminServer(logs)
    await adminServer.setup()
  }

  logs.log('Creating HOPR Node')
  let options = await generateNodeOptions()
  if (argv.dryRun) {
    console.log(JSON.stringify(options, undefined, 2))
    process.exit(0)
  }

  try {
    node = await Hopr.create(options)
    logs.log('Created HOPR Node')

    if (argv.rest) {
      const http = require('http')
      const service = require('restana')()

      service.get('/api/v1/version', (_, res) => res.send(FULL_VERSION))
      service.get('/api/v1/address/eth', async (_, res) => res.send(await node.paymentChannels.hexAccountAddress()))
      service.get('/api/v1/address/hopr', async (_, res) => res.send(await node.getId().toB58String()))
      const hostname = argv.restHost
      const port = argv.restPort
      http.createServer(service).listen(port, hostname, () => {
        logs.log(`Rest server on ${hostname} listening on port ${port}`)
      })
    }

    node.on('hopr:message', logMessageToNode)

    if (adminServer) {
      adminServer.registerNode(node)
    }

    if (argv.run && argv.run !== '') {
      // Run a single command and then exit.
      let cmds = new Commands(node)
      if (argv.settings) {
        cmds.setState(settings)
      }
      // We support multiple semicolon separated commands
      let toRun = argv.run.split(';')

      for (let c of toRun) {
        if (c === 'daemonize') {
          return
        }
        let resp = await cmds.execute(c)
        console.log(resp)
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
