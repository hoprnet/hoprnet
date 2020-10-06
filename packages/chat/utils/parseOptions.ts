import getopts from 'getopts'
import chalk from 'chalk'

import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import Multiaddr from 'multiaddr'

import ListConnctor from '../commands/listConnectors'

import { displayHelp } from './displayHelp'
import { decodeMessage } from './message'

import { knownConnectors } from './knownConnectors'
import type { HoprOptions } from '@hoprnet/hopr-core'
import { getBootstrapAddresses } from '@hoprnet/hopr-utils'

const listConnectors = new ListConnctor()

function parseHosts(): HoprOptions['hosts'] {
  const hosts: HoprOptions['hosts'] = {}

  if (process.env['HOST_IPV4'] !== undefined) {
    const str = process.env['HOST_IPV4'].replace(/\/\/.+/, '').trim()
    const params = str.match(/([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\:([0-9]{1,6})/)
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv4 host. Got ${chalk.yellow(str)}`)
    }

    hosts.ip4 = {
      ip: params[1],
      port: parseInt(params[2]),
    }
  }

  if (process.env['HOST_IPV6'] !== undefined) {
    // '\[('
    // '[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}|'
    // '[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|'
    // '[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|'
    // '[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|'
    // '[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|'
    // '\:\:[0-9a-fA-F]{1,4}|'
    // '\:\:'
    // ')\]\:'
    // '([0-9]{1,6})'

    const str = process.env['HOST_IPV6'].replace(/\/\/.+/, '').trim()
    const params = str.match(
      /\[([0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|[0-9a-fA-F]{1,4}\:\:[0-9a-fA-F]{1,4}|\:\:[0-9a-fA-F]{1,4}|\:\:)\]\:([0-9]{1,6})/
    )
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv6 host. Got ${chalk.yellow(str)}`)
    }

    hosts.ip6 = {
      ip: params[1],
      port: parseInt(params[2]),
    }
  }

  return hosts
}

/**
 * Parses the given command-line options and returns a configuration object.
 *
 * @returns
 */
export async function parseOptions(): Promise<HoprOptions> {
  const unknownOptions: string[] = []

  let cli_options = getopts(process.argv.slice(2), {
    boolean: ['debug', 'bootstrapNode', 'help', 'listConnectors', 'verbose'],
    string: ['network', 'password'],
    alias: {
      l: 'listConnectors',
      p: 'password',
      bootstrap: 'bootstrapNode',
      b: 'bootstrapNode',
      h: 'help',
      n: 'network',
      v: 'verbose',
    },
    default: {
      network: 'ethereum',
      bootstrapNode: false,
    },
    unknown: (option: string) => {
      unknownOptions.push(option)
      return false
    },
  })

  if (cli_options._.length > 1) {
    console.log(`Found more than the allowed options. Got ${chalk.yellow(cli_options._.join(', '))}\n`)
    displayHelp()
    process.exit(0)
  }

  let id: number | undefined
  for (let i = 0; i < cli_options._.length; i++) {
    try {
      const int = parseInt(cli_options._[i])

      if (isFinite(int)) {
        id = int
      }
    } catch {
      console.log(chalk.yellow(`Got unknown option '${cli_options._[i]}'.`))
      displayHelp()
      process.exit(0)
    }
  }

  if (unknownOptions.length > 0) {
    console.log(
      chalk.yellow(`Got unknown option${unknownOptions.length == 1 ? '' : 's'} [${unknownOptions.join(', ')}]\n`)
    )
    displayHelp()
    process.exit(0)
  }

  if (cli_options.verbose) {
    require('debug').enable('*')
  }

  if (cli_options.help) {
    displayHelp()
    process.exit(0)
  }

  if (cli_options.listConnectors) {
    await listConnectors.execute()
    process.exit()
  }

  if (!knownConnectors.some((connector) => connector[1] == cli_options.network)) {
    console.log(`Unknown network! <${chalk.red(cli_options.network)}>\n`)
    await listConnectors.execute()
    throw new Error('Cannot launch without a network')
  }

  let addr: Multiaddr
  let bootstrapServerMap = new Map<string, PeerInfo>()

  if (!cli_options.bootstrapNode) {
    bootstrapServerMap = await getBootstrapAddresses()
  }

  const provider = process.env[`${cli_options.network.toUpperCase()}_PROVIDER`]
  if (provider === undefined) {
    throw Error(
      `Could not find any connector for ${chalk.magenta(cli_options.network)}. Please specify ${chalk.yellow(
        `${cli_options.network.toUpperCase()}_PROVIDER`
      )} in ${chalk.yellow('.env')}.`
    )
  }

  let options: HoprOptions = {
    debug: cli_options.debug || false,
    bootstrapNode: cli_options.bootstrapNode,
    network: cli_options.network,
    bootstrapServers: [...bootstrapServerMap.values()],
    provider: provider,
    output(encoded: Uint8Array) {
      const { latency, msg } = decodeMessage(encoded)

      let str = `\n`

      str += `===== New message ======\n`
      str += `Message: ${chalk.yellow(msg.toString())}\n`
      str += `Latency: ${chalk.green(latency.toString())} ms\n`
      str += `========================\n`

      console.log(str)
    },
    hosts: parseHosts(),
  }

  if (id != null) {
    options.id = id
  }

  if (cli_options.password) {
    options.password = cli_options.password
  }

  return options
}
