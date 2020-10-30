import dotenv from 'dotenv'
// @ts-ignore
const dotenvExpand = require('dotenv-expand')
const env = dotenv.config()
dotenvExpand(env)

import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type {HoprOptions} from '@hoprnet/hopr-core'
import Hopr from '@hoprnet/hopr-core'
import {clearString} from '@hoprnet/hopr-utils'
import chalk from 'chalk'
import readline from 'readline'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import clear from 'clear'
import {parseOptions, yesOrNoQuestion} from './utils'
import {Commands} from './commands'
import {renderHoprLogo} from './logo'
import pkg from './package.json'

export * as commands from './commands'

// Name our process 'hopr'
process.title = 'hopr'

let node: Hopr<HoprCoreConnector>

async function runAsRegularNode() {
  let commands: Commands

  let rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    // See readline for explanation of this signature.
    completer: async (line: string, cb: (err: Error | undefined, hits: [string[], string]) => void) => {
      let results = await commands.autocomplete(line)
      cb(undefined, results)
    }
  })

  commands = new Commands(node, rl)

  rl.on('SIGINT', async () => {
    const question = 'Are you sure you want to exit?'
    const shouldTerminate = await yesOrNoQuestion(rl, question)

    if (shouldTerminate) {
      clearString(question, rl)
      await commands.execute('quit')
      return
    }

    rl.prompt()
  })

  rl.once('close', async () => {
    await commands.execute('quit')
    return
  })

  console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`)

  rl.on('line', async (input: string) => {
    if (input == null || input == '') {
      rl.prompt()
      return
    }

    let result = await commands.execute(input)
    if (result) {
      console.log(result)
    }
    rl.prompt()
  })

  rl.prompt()
}

function runAsBootstrapNode() {
  console.log(`... running as bootstrap node!.`)

  node.on('hopr:peer:connection', (peer: PeerId) => {
    console.log(`Incoming connection from ${chalk.blue(peer.toB58String())}.`)
  })

  process.once('exit', async () => {
    await node.stop()
    return
  })
}

async function main() {
  clear()
  renderHoprLogo()
  console.log(`Welcome to ${chalk.bold('HOPR')}!\n`)

  let options: HoprOptions
  try {
    options = await parseOptions()
  } catch (err) {
    console.log(err.message + '\n')
    return
  }

  console.log(`Chat Version: ${chalk.bold(pkg.version)}`)
  console.log(`Core Version: ${chalk.bold(pkg.dependencies['@hoprnet/hopr-core'])}`)
  console.log(`Utils Version: ${chalk.bold(pkg.dependencies['@hoprnet/hopr-utils'])}`)
  console.log(`Connector Version: ${chalk.bold(pkg.dependencies['@hoprnet/hopr-core-connector-interface'])}\n`)
  console.log(
    `Bootstrap Servers: ${chalk.bold((options.bootstrapServers || []).map((x: Multiaddr) => x.getPeerId()))}\n`
  )

  try {
    node = await Hopr.create(options)
  } catch (err) {
    console.log(chalk.red(err.message))
    process.exit(1)
  }

  console.log('Successfully started HOPR Chat.\n')
  console.log(`Your HOPR Chat node is available at the following addresses:\n ${node.getAddresses().join('\n ')}\n`)
  console.log('Use the “help” command to see which commands are available.\n')

  if (options.bootstrapNode) {
    runAsBootstrapNode()
  } else {
    runAsRegularNode()
  }
}

// If module is run as main (ie. from command line)
if (typeof module !== 'undefined' && !module.parent) {
  main()
}
