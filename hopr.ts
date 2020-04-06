import dotenv from 'dotenv'
// @ts-ignore
const dotenvExpand = require('dotenv-expand')
const env = dotenv.config()
dotenvExpand(env)

import readline from 'readline'
import getopts from 'getopts'

import chalk from 'chalk'

import groupBy from 'lodash.groupby'

import BN from 'bn.js'

import PeerInfo from 'peer-info'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import { encode, decode } from 'rlp'

// @ts-ignore
const Multihash = require('multihashes')
import bs58 from 'bs58'

import type  HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types, Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'

import Hopr from './src'
import { pubKeyToPeerId, addPubKey, u8aToHex } from './src/utils'

import figlet from 'figlet'
import clear from 'clear'

/**
 * Alphabetical list of known connectors.
 */
const knownConnectors = [
  /* prettier-ignore */
  ['@hoprnet/hopr-core-ethereum', 'ethereum'],
  ['@hoprnet/hopr-core-polkadot', 'polkadot']
]

let node: Hopr<HoprCoreConnector>,
  funds: Types.Balance,
  ownAddress: Types.AccountId,
  rl: readline.Interface,
  options: any,
  connector: typeof HoprCoreConnector

const SPLIT_OPERAND_QUERY_REGEX: RegExp = /([\w\-]+)(?:\s+)?([\w\s\-.]+)?/

/**
 * Displays an overview of possible options
 */
function displayHelp(): void {
  console.log(
    /* prettier-ignore */
    `\nStart HOPR with:\n` +
      `-b [--bootstrapNode, bootstrap]  starts HOPR as a bootstrap node\n` +
      `-n [--network]                   specifies which connector to use\n` +
      `-h [--help]                      shows this help page\n` + 
      `<ID>                             starts HOPR with ID <ID> as specified in .env\n`
  )
}

/**
 * Takes the string representation of a peerId and checks whether it is a valid
 * peerId, i. e. it is a valid base58 encoding.
 * It then generates a PeerId instance and returns it.
 *
 * @param query query that contains the peerId
 */
async function checkPeerIdInput(query: string): Promise<PeerId> {
  let peerId: PeerId

  try {
    // Throws an error if the Id is invalid
    Multihash.decode(bs58.decode(query))

    peerId = await addPubKey(PeerId.createFromB58String(query))
  } catch (err) {
    throw Error(chalk.red(`Invalid peerId. ${err.message}`))
  }

  return peerId
}

/**
 * Checks whether the given PeerId belongs to any known bootstrap node.
 *
 * @param peerId
 */
function isBootstrapNode(node: Hopr<HoprCoreConnector>, peerId: PeerId): boolean {
  for (let i = 0; i < node.bootstrapServers.length; i++) {
    if (peerId.isEqual(node.bootstrapServers[i].id)) {
      return true
    }
  }
  return false
}

interface Options {
  network: string
  bootstrapNode: boolean
  id?: number
}

/**
 * Parses the given command-line options and returns a configuration object.
 *
 * @returns
 */
async function parseOptions(): Promise<void | Options> {
  const unknownOptions: string[] = []

  const options = getopts(process.argv.slice(2), {
    boolean: ['bootstrapNode', 'help'],
    string: ['network'],
    alias: {
      bootstrap: 'bootstrapNode',
      b: 'bootstrapNode',
      h: 'help',
      n: 'network'
    },
    default: {
      network: 'ethereum',
      bootstrapNode: false
    },
    unknown: (option: string) => {
      unknownOptions.push(option)
      return false
    }
  })

  delete options.b
  delete options.n

  for (let i = 0; i < options._.length; i++) {
    try {
      const int = parseInt(options._[i])

      if (isFinite(int)) {
        options.id = int
      }
    } catch {}
  }

  delete options._

  if (unknownOptions.length > 0) {
    console.log(`Got unknown option${unknownOptions.length == 1 ? '' : 's'} [${unknownOptions.join(', ')}]`)
    return
  }

  if (!knownConnectors.some(connector => connector[1] == options.network)) {
    console.log(`Unknown network! <${chalk.red(options.network)}>\n`)
    await listConnectors()
    return
  }

  try {
    connector = (await import(`@hoprnet/hopr-core-${options.network}`)).default as typeof HoprCoreConnector
  } catch (err) {
    console.log(`Could not find <${chalk.red(`@hoprnet/hopr-core-${options.network}`)}>. Please make sure it is available under ./node_modules!\n`)
    await listConnectors()
    return
  }

  const tmp = groupBy(
    process.env.BOOTSTRAP_SERVERS?.split(',').map(addr => Multiaddr(addr)),
    (ma: Multiaddr) => ma.getPeerId()
  )

  options.bootstrapServers = []
  for (const peerId of Object.keys(tmp)) {
    const peerInfo = new PeerInfo(PeerId.createFromB58String(peerId))

    tmp[peerId].forEach((ma: Multiaddr) => peerInfo.multiaddrs.add(ma))
    options.bootstrapServers.push(peerInfo)
  }

  options.provider = process.env.ETHEREUM_PROVIDER

  options.output = (encoded: Uint8Array) => {
    const { latency, msg } = decodeMessage(encoded)

    let str = `\n`

    str += `===== New message ======\n`
    str += `Message: ${chalk.yellow(msg.toString())}\n`
    str += `Latency: ${chalk.green(latency.toString())} ms\n`
    str += `========================\n`

    console.log(str)
  }

  return (options as unknown) as Options
}

// Allowed keywords
const keywords = ['open', 'stake', 'stakedEther', 'unstake', 'send', 'quit', 'crawl', 'openChannels', 'closeAll', 'myAddress', 'balance', 'listConnectors', 'ping']

/**
 * Completes a given input with possible endings. Used for convenience.
 *
 * @param line the current input
 * @param cb to returns the suggestions
 */
async function tabCompletion(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void) {
  if (line == null || line == '') {
    return cb(undefined, [keywords, line])
  }

  const [command, query]: (string | undefined)[] = line
    .trim()
    .split(SPLIT_OPERAND_QUERY_REGEX)
    .slice(1)

  switch (command) {
    case 'send':
      const peerInfos: PeerInfo[] = []
      for (const peerInfo of node.peerStore.peers.values()) {
        if ((!query || peerInfo.id.toB58String().startsWith(query)) && !isBootstrapNode(node, peerInfo.id)) {
          peerInfos.push(peerInfo)
        }
      }

      if (!peerInfos.length) {
        console.log(chalk.red(`\nDoesn't know any other node except apart from bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}!`))
        return cb(undefined, [[''], line])
      }

      return cb(undefined, [peerInfos.map((peerInfo: PeerInfo) => `send ${peerInfo.id.toB58String()}`), line])
    case 'open':
      node.paymentChannels.channel.getAll(
        node.paymentChannels,
        async (channel: ChannelInstance<HoprCoreConnector>) => (await pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(),
        async (channelIds: Promise<string>[]) => {
          let peerIdStringSet: Set<string>

          try {
            peerIdStringSet = new Set<string>(await Promise.all(channelIds))
          } catch (err) {
            console.log(chalk.red(err.message))
            return cb(undefined, [[''], line])
          }

          const peers: string[] = []
          for (const peerInfo of node.peerStore.peers.values()) {
            if (isBootstrapNode(node, peerInfo.id)) {
              continue
            }

            if (!peerIdStringSet.has(peerInfo.id.toB58String())) {
              peers.push(peerInfo.id.toB58String())
            }
          }

          if (peers.length < 1) {
            console.log(chalk.red(`\nDoesn't know any node to open a payment channel with.`))
            return cb(undefined, [[''], line])
          }

          const hits = query ? peers.filter((peerId: string) => peerId.startsWith(query)) : peers

          return cb(undefined, [hits.length ? hits.map((str: string) => `open ${str}`) : ['open'], line])
        }
      )
      break
    case 'close':
      node.paymentChannels.channel.getAll(
        node.paymentChannels,
        async (channel: ChannelInstance<HoprCoreConnector>) => (await pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(),
        async (peerIdPromises: Promise<string>[]) => {
          let peerIdStrings: string[]
          try {
            peerIdStrings = await Promise.all(peerIdPromises)
          } catch (err) {
            console.log(chalk.red(err.message))
            return cb(undefined, [[''], line])
          }

          if (peerIdStrings != null && peerIdStrings.length < 1) {
            console.log(chalk.red(`\nCannot close any channel because there are not any open ones and/or channels were opened by a third party.`))
            return cb(undefined, [[''], line])
          }

          const hits = query ? peerIdStrings.filter((peerId: string) => peerId.startsWith(query)) : peerIdStrings

          return cb(undefined, [hits.length ? hits.map((str: string) => `close ${str}`) : ['close'], line])
        }
      )
      break
    case 'ping': {
      const peerInfos: PeerInfo[] = []
      for (const peerInfo of node.peerStore.peers.values()) {
        if ((!query || peerInfo.id.toB58String().startsWith(query)) && !isBootstrapNode(node, peerInfo.id)) {
          peerInfos.push(peerInfo)
        }
      }

      if (!peerInfos.length) {
        console.log(chalk.red(`\nDoesn't know any other node except apart from bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}!`))
        return cb(undefined, [[''], line])
      }

      return cb(undefined, [peerInfos.map((peerInfo: PeerInfo) => `ping ${peerInfo.id.toB58String()}`), line])
    }
    default:
      const hits = keywords.filter(keyword => keyword.startsWith(line))

      return cb(undefined, [hits.length ? hits : keywords, line])
  }
}

/**
 * Stops the node and kills the process in case it does not quit by itself.
 */
async function stopNode(): Promise<void> {
  if (node == null) {
    return
  }

  const timeout = setTimeout(() => {
    console.log(`Ungracefully stopping node after timeout.`)
    process.exit(0)
  }, 10 * 1000)

  try {
    await node.stop()
    clearTimeout(timeout)
  } catch (err) {
    console.log(chalk.red(err.message))
  }
}

function runAsBootstrapNode() {
  if (options.bootstrapNode) {
    console.log(`... running as bootstrap node!.`)
  }

  node.on('peer:connect', (peer: PeerInfo) => {
    console.log(`Incoming connection from ${chalk.blue(peer.id.toB58String())}.`)
  })

  rl.resume()
}

async function runAsRegularNode() {
  ownAddress = await node.paymentChannels.utils.pubKeyToAccountId(node.peerInfo.id.pubKey.marshal())

  try {
    funds = await node.paymentChannels.accountBalance
  } catch (err) {
    console.log(chalk.red(err.message))
    return stopNode()
  }

  console.log(`Connecting to bootstrap node${node.bootstrapServers.length == 1 ? '' : 's'}...`)

  rl.on('line', async (input: string) => {
    if (input == null || input == '') {
      console.log(chalk.red('Unknown command!'))
      rl.prompt()
      return
    }

    rl.pause()
    const [command, query]: (string | undefined)[] = input
      .trim()
      .split(SPLIT_OPERAND_QUERY_REGEX)
      .slice(1)

    switch ((command || '').trim()) {
      case 'crawl':
        crawl()
        break
      case 'quit':
        await stopNode()
        process.exit(0)
      case 'balance':
        printBalance()
        break
      // case 'stake':
      //   stake(query)
      //   break
      // case 'stakedEther':
      //   let tmp = await getStakedEther()

      //   if (tmp) stakedFunds = tmp
      //   break
      // case 'unstake':
      //   unstake(query)
      //   break
      case 'openChannels':
        openChannels()
        break
      case 'open':
        open(query)
        break
      case 'send':
        send(query)
        break
      // case 'closeAll':
      //   closeAll()
      //   break
      case 'close':
        close(query)
        break
      case 'listConnectors':
        listConnectors()
        break
      case 'myAddress':
        printMyAddress()
        break
      case 'ping':
        ping(query)
        break
      default:
        console.log(chalk.red('Unknown command!'))
        rl.prompt()
        break
    }
  })

  rl.prompt()
}

process.title = 'hopr'

async function main() {
  clear()
  console.log(
    figlet.textSync('HOPR.network', {
      horizontalLayout: 'fitted'
    })
  )
  console.log(`Welcome to ${chalk.bold('HOPR')}!\n`)

  options = await parseOptions()

  if (options == null || options.help) {
    displayHelp()
    return
  }

  try {
    node = await Hopr.createNode(connector, options)
  } catch (err) {
    await stopNode()
    console.log(chalk.red(err.message))
    process.exit(1)
  }
  
  rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    completer: tabCompletion
  })

  rl.once('close', async () => {
    await stopNode()
    process.exit(0)
  })

  console.log(`\nAvailable under the following addresses:\n ${node.peerInfo.multiaddrs.toArray().join('\n ')}\n`)

  if (options.bootstrapNode) {
    runAsBootstrapNode()
  } else {
    runAsRegularNode()
  }
}

main()

async function close(query?: string): Promise<any> {
  if (query == null) {
    console.log(chalk.red(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`))
    setTimeout(() => {
      rl.prompt()
    })
    return
  }

  let peerId: PeerId
  try {
    peerId = await checkPeerIdInput(query)
  } catch (err) {
    console.log(err.message)
    setTimeout(() => {
      rl.prompt()
    })
    return
  }

  const unsubscribe = startDelayedInterval(`Initiated settlement. Waiting for finalisation`)

  let channel: ChannelInstance<HoprCoreConnector>

  try {
    channel = await node.paymentChannels.channel
      .create(node.paymentChannels, peerId.pubKey.marshal(), async (counterparty: Uint8Array) => node.interactions.payments.onChainKey.interact(await pubKeyToPeerId(counterparty)))
  
    await channel.initiateSettlement()

    console.log(`${chalk.green(`Successfully closed channel`)} ${chalk.yellow(u8aToHex(await channel.channelId))}. Received ${chalk.magenta(new BN(0).toString())} ${node.paymentChannels.types.Balance.SYMBOL}.`)
  } catch (err) {
    console.log(chalk.red(err.message))
  } finally {
    setTimeout(() => {
      unsubscribe()
      process.stdout.write('\n')
      rl.prompt()
    })
  }
}

/**
 * Encapsulates the functionality that is executed once the user decides to send a message.
 * @param query peerId string to send message to
 */
async function send(query?: string): Promise<void> {
  if (query == null) {
    console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`))
    setTimeout(() => {
      rl.prompt()
    })
    return
  }

  let peerId: PeerId
  try {
    peerId = await checkPeerIdInput(query)
  } catch (err) {
    console.log(chalk.red(err.message))
    setTimeout(() => {
      rl.prompt()
    })
    return
  }

  rl.question(`Sending message to ${chalk.blue(peerId.toB58String())}\nType in your message and press ENTER to send:\n`, async (message: string) => {
    try {
      await node.sendMessage(encodeMessage(message), peerId)

      setTimeout(() => {
        rl.prompt()
      })
    } catch (err) {
      console.log(chalk.red(err.message))

      setTimeout(() => {
        rl.prompt()
      })
    }
  })
}

/**
 * Encapsulates the functionality that is executed once the user decides to open a payment channel
 * with another party.
 * @param query peerId string to send message to
 */
async function open(query?: string): Promise<void> {
  if (query == null || query == '') {
    console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`))
    rl.prompt()
    return
  }

  let counterparty: PeerId
  try {
    counterparty = await checkPeerIdInput(query)
  } catch (err) {
    setTimeout(() => {
      console.log(err.message)
      rl.prompt()
    })
    return
  }

  const channelId = await node.paymentChannels.utils.getId(
    /* prettier-ignore */
    await node.paymentChannels.utils.pubKeyToAccountId(node.peerInfo.id.pubKey.marshal()),
    await node.paymentChannels.utils.pubKeyToAccountId(counterparty.pubKey.marshal())
  )

  const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)

  try {
    await node.paymentChannels.channel.create(
      node.paymentChannels,
      counterparty.pubKey.marshal(),
      async () => node.paymentChannels.utils.pubKeyToAccountId(await node.interactions.payments.onChainKey.interact(counterparty)),
      node.paymentChannels.types.ChannelBalance.create(undefined, {
        balance: new BN(12345),
        balance_a: new BN(123)
      }),
      (balance: Types.ChannelBalance): Promise<Types.SignedChannel<Types.Channel, Types.Signature>> => node.interactions.payments.open.interact(counterparty, balance)
    )

    console.log(`${chalk.green(`Successfully opened channel`)} ${chalk.yellow(u8aToHex(channelId))}`)
  } catch (err) {
    console.log(chalk.red(err.message))
  } finally {
    setTimeout(() => {
      unsubscribe()
      rl.prompt()
    })
  }
}

/**
 * Lists all channels that we have with other nodes. Triggered from the CLI.
 */
async function openChannels(): Promise<void> {
  let str = `${chalk.yellow('ChannelId:'.padEnd(66, ' '))} - ${chalk.blue('PeerId:')}\n`

  try {
    await node.paymentChannels.channel.getAll(
      node.paymentChannels,
      async (channel: ChannelInstance<HoprCoreConnector>) => {
        const channelId = await channel.channelId
        if (!channel.counterparty) {
          str += `${chalk.yellow(u8aToHex(channelId))} - ${chalk.gray('pre-opened')}`
          return
        }

        const peerId = await pubKeyToPeerId(await channel.offChainCounterparty)

        str += `${chalk.yellow(u8aToHex(channelId))} - ${chalk.blue(peerId.toB58String())}\n`
        return
      },
      async (promises: Promise<void>[]) => {
        if (promises.length == 0) {
          str += `  No open channels.`
          return
        }

        await Promise.all(promises)

        return
      }
    )
    setTimeout(() => {
      console.log(str)
      rl.prompt()
    })
  } catch (err) {
    setTimeout(() => {
      console.log(chalk.red(err.message))
      rl.prompt()
    })
    return
  }
}

// async function stake(query: string): Promise<void> {
//   if (!query) {
//     console.log(chalk.red(`Invalid arguments. Expected 'stake <amount of ETH>'. Received '${query}'`))
//     rl.prompt()
//     return
//   }

//   let amount = new BN(toWei(query, 'ether'))
//   if (funds.lt(new BN(amount))) {
//     console.log(chalk.red('Insufficient funds.'))
//     rl.prompt()
//     return
//   }

//   try {
//     // @TODO
//     // await sendTransaction(
//     //   {
//     //     from: ownAddress,
//     //     to: process.env['CONTRACT_ADDRESS'],
//     //     value: amount.toString(),
//     //     gas: STAKE_GAS_AMOUNT
//     //   },
//     //   node.peerInfo.id,
//     //   node.paymentChannels.web3
//     // ).then(receipt => {
//     //   node.paymentChannels.nonce = node.paymentChannels.nonce + 1
//     //   return receipt
//     // })
//   } catch (err) {
//     console.log(chalk.red(err.message))
//   } finally {
//     setTimeout(() => {
//       rl.prompt()
//     })
//   }
// }

// async function unstake(query: string): Promise<void> {
//   if (query == null) {
//     console.log(chalk.red(`Invalid arguments. Expected 'unstake <amount of ETH>'. Received '${query}'`))
//     rl.prompt()
//     return
//   }

//   let amount = new BN(toWei(query, 'ether'))
//   if (stakedFunds.lt(amount)) {
//     console.log(chalk.red('Amount must not be higher than current stake.'))
//     rl.prompt()
//     return
//   }

//   try {
//     // x    await node.paymentChannels.contractCall(node.paymentChannels.contract.methods.unstakeEther(amount.toString()))
//   } catch (err) {
//     console.log(chalk.red(err.message))
//   } finally {
//     setTimeout(() => {
//       rl.prompt()
//     })
//   }
// }

/**
 * Crawls the network to check for other nodes. Triggered by the CLI.
 */
async function crawl(): Promise<void> {
  try {
    await node.network.crawler.crawl((peerInfo: PeerInfo) => !isBootstrapNode(node, peerInfo.id))
  } catch (err) {
    console.log(chalk.red(err.message))
  } finally {
    setTimeout(() => {
      rl.prompt()
    })
  }
}

// async function closeAll(): Promise<void> {
//   try {
//     const receivedMoney = await node.paymentChannels.channel.closeChannels(node.paymentChannels)
//     console.log(`${chalk.green(`Closed all channels and received`)} ${chalk.magenta(fromWei(receivedMoney.toString(), 'ether'))} ETH.`)
//   } catch (err) {
//     console.log(chalk.red(err.message))
//   } finally {
//     setTimeout(() => {
//       rl.prompt()
//     })
//   }
// }

// async function getStakedEther(): Promise<BN> {
//   try {
//     // let state = await node.paymentChannels.contract.methods.states(ownAddress).call({ from: ownAddress })
//     // console.log(`Current stake: ${chalk.green(fromWei(state.stakedEther, 'ether'))} ETH`)
//     // return new BN(state.stakedEther)
//   } catch (err) {
//     console.log(chalk.red(err.message))
//   } finally {
//     setTimeout(() => {
//       rl.prompt()
//     })
//   }

//   return Promise.resolve(new BN(0))
// }

/**
 * Prints the name of the network we are using and the
 * identity that we have on that chain.
 * @notice triggered by the CLI
 */
async function printMyAddress(): Promise<void> {
  const prefixLength = Math.max(connector.constants.CHAIN_NAME.length, 'HOPR'.length) + 3

  console.log(
    `${(connector.constants.CHAIN_NAME + ':').padEnd(prefixLength, ' ')}${chalk.green(
      bs58.encode(await node.paymentChannels.utils.pubKeyToAccountId(node.peerInfo.id.pubKey.marshal()))
    )}\n` +
      /* prettier-ignore */
      `${'HOPR:'.padEnd(prefixLength, ' ')}${chalk.green(node.peerInfo.id.toB58String())}`
  )
  setTimeout(() => rl.prompt())
}

/**
 * Prints the balance of our account.
 * @notice triggered by the CLI
 */
async function printBalance(): Promise<void> {
  console.log(`Account Balance:  `, chalk.magenta((await node.paymentChannels.accountBalance).toString()), `HOPR tokens`)
  setTimeout(() => rl.prompt())
}

/**
 * Check which connectors are present right now.
 * @notice triggered by the CLI
 */
async function listConnectors(): Promise<void> {
  let str = 'Available connectors:'
  let found = 0

  const promises = []
  for (let i = 0; i < knownConnectors.length; i++) {
    promises.push(
      import(knownConnectors[i][0]).then(
        () => {
          found++
          str += `\n  ${chalk.yellow(knownConnectors[i][0])} ${chalk.gray('=>')} ts-node hopr -n ${chalk.green(knownConnectors[i][1])}`
        },
        () => {}
      )
    )
  }

  await Promise.all(promises)

  if (found > 0) {
    console.log(str)
  } else {
    console.log(chalk.red(`Could not find any connectors. Please make sure there is one available in 'node_modules'!`))
  }

  if (rl != null) {
    setTimeout(() => rl.prompt())
  }
}

async function ping(query?: string): Promise<void> {
  if (query == null) {
    console.log(chalk.red(`Invalid arguments. Expected 'ping <peerId>'. Received '${query}'`))
    setTimeout(() => {
      rl.prompt()
    })
    return
  }

  let peerId: PeerId
  try {
    peerId = await checkPeerIdInput(query)
  } catch (err) {
    console.log(chalk.red(err.message))
    setTimeout(() => {
      rl.prompt()
    })
    return
  }

  try {
    const latency = await node.ping(peerId)
    console.log(`Pong received in:`, chalk.magenta(String(latency)), `ms`)

    setTimeout(() => {
      rl.prompt()
    })
  } catch (err) {
    console.log(`Could not ping node. Error was: ${chalk.red(err.message)}`)

    setTimeout(() => {
      rl.prompt()
    })
  }
}

/**
 * Adds the current timestamp to the message in order to measure the latency.
 * @param msg the message
 */
function encodeMessage(msg: string): Uint8Array {
  return encode([msg, Date.now()])
}

/**
 * Tries to decode the message and returns the message as well as
 * the measured latency.
 * @param encoded an encoded message
 */
function decodeMessage(
  encoded: Uint8Array
): {
  latency: number
  msg: string
} {
  let msg: Buffer, time: Buffer
  try {
    [msg, time] = decode(encoded) as [Buffer, Buffer]

    return {
      latency: Date.now() - parseInt(time.toString('hex'), 16),
      msg: msg.toString()
    }
  } catch (err) {
    console.log(chalk.red(`Could not decode received message '${u8aToHex(encoded)}' Error was ${err.message}.`))

    return {
      latency: NaN,
      msg: 'Error: Could not decode message'
    }
  }
}

function startDelayedInterval(msg: string): () => void {
  let interval: NodeJS.Timeout
  let timeout = setTimeout(() => {
    process.stdout.write(`${chalk.green(msg)}\n`)
    interval = setInterval(() => {
      process.stdout.write(chalk.green('.'))
    }, 1000)
  }, 2 * 1000)

  return () => {
    clearTimeout(timeout)
    clearInterval(interval)
  }
}
