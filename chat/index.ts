import dotenv from 'dotenv'
// @ts-ignore
const dotenvExpand = require('dotenv-expand')
const packageJSON = require('./package.json')

const env = dotenv.config()
dotenvExpand(env)

import chalk from 'chalk'

import readline from 'readline'

import PeerInfo from 'peer-info'

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import Hopr from '@hoprnet/hopr-core'
import type { HoprOptions } from '@hoprnet/hopr-core'

import figlet from 'figlet'
import clear from 'clear'

import { parseOptions } from './utils'
import { clearString } from '@hoprnet/hopr-utils'
import Commands from './commands'

const SPLIT_OPERAND_QUERY_REGEX: RegExp = /([\w\-]+)(?:\s+)?([\w\s\-.]+)?/

// Allowed keywords
export const keywords: string[][] = [
  ['open', 'opens a payment channel'],
  ['send', 'sends a message to another party'],
  ['quit', 'stops the node and terminates the process'],
  ['crawl', 'crawls the network and tries to find other nodes'],
  ['openChannels', 'lists all currently open channels'],
  ['closeAll', 'closes all payment channel of this node'],
  ['myAddress', 'shows the address of this node'],
  ['balance', 'shows our current balance'],
  ['listConnectors', 'lists all installed blockchain connectors'],
  ['ping', 'pings another node to check its availability'],
  ['version', 'shows the versions for `hopr-chat` and `hopr-core`'],
  ['help', 'shows this help page'],
  ['tickets', 'lists tickets of a channel'],
].sort((a, b) => a[0].localeCompare(b[0], 'en', { sensitivity: 'base' }))

// Allowed CLI options
export const cli_options: string[][] = [
  ['-b', '--bootstrapNode', undefined, 'starts HOPR as a bootstrap node'],
  ['-n', '--network', '<connector>', 'starts HOPR with blockchain connector <connector>'],
  ['-h', '--help', undefined, 'shows this help page'],
  ['-l', '--listConnectors', undefined, 'shows all available connectors'],
  ['-p', '--password', '<password>', 'start HOPR with <password>'],
  ['-v', '--verbose', undefined, 'show debug info'],
  [undefined, '--debug', undefined, 'run HOPR in debug mode [insecure, only used for development]'],
  // ['<ID>', undefined, undefined, 'starts HOPR with a demo ID'],
].sort((a, b) => {
  let tmpA: string
  let tmpB: string
  if (a[0] === undefined) {
    tmpA = a[1].slice(2)
  } else {
    tmpA = a[0].slice(1)
  }

  if (b[0] === undefined) {
    tmpB = b[1].slice(2)
  } else {
    tmpB = b[0].slice(1)
  }
  return tmpA.localeCompare(tmpB, 'en', { sensitivity: 'base' })
})

// Name our process 'hopr'
process.title = 'hopr'

/**
 * Alphabetical list of known connectors.
 */
export const knownConnectors = [
  /* prettier-ignore */
  ['@hoprnet/hopr-core-ethereum', 'ethereum'],
  ['@hoprnet/hopr-core-polkadot', 'polkadot'],
]

let node: Hopr<HoprCoreConnector>

/**
 * Completes a given input with possible endings. Used for convenience.
 *
 * @param line the current input
 * @param cb to returns the suggestions
 */
function tabCompletion(commands: Commands) {
  return async (line: string, cb: (err: Error | undefined, hits: [string[], string]) => void) => {
    if (line == null || line == '') {
      return cb(undefined, [keywords.map(entry => entry[0]), line])
    }

    const [command, query]: (string | undefined)[] = line.trim().split(SPLIT_OPERAND_QUERY_REGEX).slice(1)

    if (command == null || command === '') {
      return cb(undefined, [keywords.map(entry => entry[0]), line])
    }

    switch (command.trim()) {
      case 'send':
        await commands.sendMessage.complete(line, cb, query)
        break
      case 'open':
        await commands.openChannel.complete(line, cb, query)
        break
      case 'close':
        commands.closeChannel.complete(line, cb, query)
        break
      case 'ping': {
        commands.ping.complete(line, cb, query)
        break
      }
      case 'tickets': {
        await commands.tickets.complete(line, cb, query)
      }
      default:
        const hits = keywords.reduce((acc: string[], keyword: [string, string]) => {
          if (keyword[0].startsWith(line)) {
            acc.push(keyword[0])
          }

          return acc
        }, [])

        return cb(undefined, [hits.length ? hits : keywords.map(keyword => keyword[0]), line])
    }
  }
}

async function runAsRegularNode() {
  const commands = new Commands(node)

  let rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    completer: tabCompletion(commands),
  })

  rl.on('SIGINT', async () => {
    const question = `Are you sure you want to exit? (${chalk.green('y')}, ${chalk.red('N')}): `

    const answer = await new Promise<string>(resolve => rl.question(question, resolve))

    if (answer.match(/^y(es)?$/i)) {
      clearString(question, rl)
      await commands.stopNode.execute()
      return
    }
    rl.prompt()
  })

  rl.once('close', async () => {
    await commands.stopNode.execute()
    return
  })

  console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`)

  rl.on('line', async (input: string) => {
    if (input == null || input == '') {
      console.log(chalk.red('Unknown command!'))
      rl.prompt()
      return
    }

    const [command, query]: (string | undefined)[] = input.trim().split(SPLIT_OPERAND_QUERY_REGEX).slice(1)

    if (command == null) {
      console.log(chalk.red('Unknown command!'))
      rl.prompt()
      return
    }

    switch (command.trim()) {
      case 'balance':
        await commands.printBalance.execute()
        break
      case 'close':
        await commands.closeChannel.execute(query)
        break
      case 'crawl':
        await commands.crawl.execute()
        break
      case 'help':
        commands.listCommands.execute()
        break
      case 'quit':
        await commands.stopNode.execute()
        break
      case 'openChannels':
        await commands.listOpenChannels.execute()
        break
      case 'open':
        await commands.openChannel.execute(rl, query)
        break
      case 'send':
        await commands.sendMessage.execute(rl, query)
        break

      case 'listConnectors':
        await commands.listConnectors.execute()
        break
      case 'myAddress':
        await commands.printAddress.execute()
        break
      case 'ping':
        await commands.ping.execute(query)
        break
      case 'version':
        await commands.version.execute()
        break
      case 'tickets':
        await commands.tickets.execute(query)
        break
      default:
        console.log(chalk.red('Unknown command!'))
        break
    }

    rl.prompt()
  })

  rl.prompt()
}

function runAsBootstrapNode() {
  console.log(`... running as bootstrap node!.`)

  node.on('peer:connect', (peer: PeerInfo) => {
    console.log(`Incoming connection from ${chalk.blue(peer.id.toB58String())}.`)
  })

  process.once('exit', async () => {
    await node.down()
    return
  })
}

async function main() {
  clear()
  console.log(
    figlet.textSync('HOPRnet.eth', {
      horizontalLayout: 'fitted',
    })
  )
  console.log(`Welcome to ${chalk.bold('HOPR')}!\n`)

  console.log(`Chat Version: ${chalk.bold(packageJSON.version)}`)
  console.log(`Core Version: ${chalk.bold(packageJSON.dependencies['@hoprnet/hopr-core'])}`)
  console.log(`Core Ethereum Version: ${chalk.bold(packageJSON.dependencies['@hoprnet/hopr-core-ethereum'])}`)
  console.log(`Utils Version: ${chalk.bold(packageJSON.dependencies['@hoprnet/hopr-utils'])}`)
  console.log(`Connector Version: ${chalk.bold(packageJSON.dependencies['@hoprnet/hopr-core-connector-interface'])}\n`)

  console.log(`Bootstrap Servers: ${chalk.bold(process.env['BOOTSTRAP_SERVERS'])}\n`)

  let options: HoprOptions
  try {
    options = await parseOptions()
  } catch (err) {
    console.log(err.message + '\n')
    return
  }

  try {
    node = await Hopr.create(options)
  } catch (err) {
    console.log(chalk.red(err.message))
    process.exit(1)
  }

  console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)

  if (options.bootstrapNode) {
    runAsBootstrapNode()
  } else {
    runAsRegularNode()
  }
}

main()
