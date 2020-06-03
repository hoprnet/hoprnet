import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type AbstractCommand from './abstractCommand'

import chalk from 'chalk'

import type PeerId from 'peer-id'
import type PeerInfo from 'peer-info'

import { checkPeerIdInput, encodeMessage, isBootstrapNode } from '../utils'
import { clearString } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import { MAX_HOPS } from '@hoprnet/hopr-core/lib/constants'

import readline from 'readline'

const getOpenChannels = async (node: Hopr<HoprCoreConnector>) => {
  return new Promise<string[]>((resolve, reject) => {
    let openChannels: string[] = []

    try {
      node.paymentChannels.channel.getAll(
        node.paymentChannels,
        async (channel: ChannelInstance<HoprCoreConnector>) => {
          const peerId = await pubKeyToPeerId(await channel.offChainCounterparty)
          const peerIdStr = peerId.toB58String()

          if (!openChannels.includes(peerIdStr)) {
            openChannels.push(peerIdStr)
          }

          return
        },
        async (promises: Promise<void>[]) => {
          await Promise.all(promises)
          return resolve(openChannels)
        }
      )
    } catch (err) {
      return reject(err)
    }
  })
}

export default class SendMessage implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * Encapsulates the functionality that is executed once the user decides to send a message.
   * @param query peerId string to send message to
   */
  async execute(rl: readline.Interface, query?: string): Promise<void> {
    if (query == null) {
      console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`))
      return
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query)
    } catch (err) {
      console.log(chalk.red(err.message))
      return
    }

    // @ts-ignore
    const oldCompleter = rl.completer
    // @ts-ignore
    rl.completer = undefined

    const manualIntermediateNodesQuestion = `Do you want to manually set the intermediate nodes? (${chalk.green('y')}, ${chalk.red('N')}): `
    const manualIntermediateNodesAnswer = await new Promise<string>(resolve => rl.question(manualIntermediateNodesQuestion, resolve))

    clearString(manualIntermediateNodesQuestion + manualIntermediateNodesAnswer, rl)

    const manualPath = (manualIntermediateNodesAnswer.toLowerCase().match(/^y(es)?$/i) || '').length

    const messageQuestion = `${chalk.yellow(`Type in your message and press ENTER to send:`)}\n`
    const message = await new Promise<string>(resolve => rl.question(messageQuestion, resolve))

    clearString(messageQuestion + message, rl)

    console.log(`Sending message to ${chalk.blue(query)} ...`)

    try {
      if (manualPath) {
        await this.node.sendMessage(encodeMessage(message), peerId, () => this.selectIntermediateNodes(rl, query))
      } else {
        await this.node.sendMessage(encodeMessage(message), peerId)
      }
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }

  async complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string): Promise<void> {
    const peerInfos: PeerInfo[] = []
    for (const peerInfo of this.node.peerStore.peers.values()) {
      if ((!query || peerInfo.id.toB58String().startsWith(query)) && !isBootstrapNode(this.node, peerInfo.id)) {
        peerInfos.push(peerInfo)
      }
    }

    if (!peerInfos.length) {
      console.log(chalk.red(`\nDoesn't know any other node except apart from bootstrap node${this.node.bootstrapServers.length == 1 ? '' : 's'}!`))
      return cb(undefined, [[''], line])
    }

    return cb(undefined, [peerInfos.map((peerInfo: PeerInfo) => `send ${peerInfo.id.toB58String()}`), line])
  }

  async selectIntermediateNodes(rl: readline.Interface, destination: string): Promise<PeerId[]> {
    console.log(chalk.yellow('Please select the intermediate nodes: (hint use tabCompletion)'))

    const openChannels = await getOpenChannels(this.node)
    let localPeers: string[] = []
    for (let peer of this.node.peerStore.peers.values()) {
      let peerIdString = peer.id.toB58String()
      if (peerIdString !== destination && openChannels.includes(peerIdString)) {
        localPeers.push(peerIdString)
      }
    }

    if (localPeers.length === 0) {
      console.log(chalk.yellow('Cannot find peers in which you have open payment channels with.'))
    }

    // @ts-ignore
    const oldPrompt = rl._prompt
    // @ts-ignore
    const oldCompleter = rl.completer

    const oldListeners = rl.listeners('line')
    rl.removeAllListeners('line')

    rl.setPrompt('')
    // @ts-ignore
    rl.completer = (line: string, cb: (err: Error | undefined, hits: [string[], string]) => void) => {
      return cb(undefined, [localPeers.filter(localPeer => localPeer.startsWith(line)), line])
    }

    let selected: PeerId[] = []

    await new Promise(resolve =>
      rl.on('line', async query => {
        if (query == null || query === '\n' || query === '' || query.length == 0) {
          rl.removeAllListeners('line')
          return resolve()
        }
        let peerId: PeerId
        try {
          peerId = await checkPeerIdInput(query)
        } catch (err) {
          console.log(chalk.red(err.message))
        }

        const peerIndex = localPeers.findIndex((str: string) => str == query)

        readline.moveCursor(process.stdout, -rl.line, -1)
        readline.clearLine(process.stdout, 0)

        console.log(chalk.blue(query))

        selected.push(peerId)
        localPeers.splice(peerIndex, 1)

        if (selected.length >= MAX_HOPS - 1) {
          rl.removeAllListeners('line')
          return resolve()
        }
      })
    )

    rl.setPrompt(oldPrompt)
    // @ts-ignore
    rl.completer = oldCompleter

    // @ts-ignore
    oldListeners.forEach(oldListener => rl.on('line', oldListener))

    return selected
  }
}
