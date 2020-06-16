import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type AbstractCommand from './abstractCommand'

import chalk from 'chalk'

import type PeerId from 'peer-id'
import type PeerInfo from 'peer-info'

import { checkPeerIdInput, encodeMessage, isBootstrapNode, getOpenChannels, getPeers } from '../utils'
import { clearString } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '@hoprnet/hopr-core/lib/constants'

import readline from 'readline'

export default class SendMessage implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * Encapsulates the functionality that is executed once the user decides to send a message.
   * @param query peerId string to send message to
   */
  async execute(rl: readline.Interface, query?: string): Promise<void> {
    if (query == null) {
      console.log(chalk.red(`Invalid arguments. Expected 'send <peerId>'. Received '${query}'`))
      return
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query)
    } catch (err) {
      console.log(chalk.red(err.message))
      return
    }

    const manualPath = process.env.MULTIHOP
      ? await (async () => {
          const manualIntermediateNodesQuestion = `Do you want to manually set the intermediate nodes? (${chalk.green(
            'y'
          )}, ${chalk.red('N')}): `
          const manualIntermediateNodesAnswer = await new Promise<string>(resolve =>
            rl.question(manualIntermediateNodesQuestion, resolve)
          )

          clearString(manualIntermediateNodesQuestion + manualIntermediateNodesAnswer, rl)

          return (manualIntermediateNodesAnswer.toLowerCase().match(/^y(es)?$/i) || '').length >= 1
        })()
      : true

    const messageQuestion = `${chalk.yellow(`Type in your message and press ENTER to send:`)}\n`
    const message = await new Promise<string>(resolve => rl.question(messageQuestion, resolve))

    clearString(messageQuestion + message, rl)

    console.log(`Sending message to ${chalk.blue(query)} ...`)

    try {
      if (manualPath) {
        await this.node.sendMessage(encodeMessage(message), peerId, async () => {
          if (process.env.MULTIHOP) return this.selectIntermediateNodes(rl, peerId)
          return []
        })
      } else {
        await this.node.sendMessage(encodeMessage(message), peerId)
      }
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }

  async complete(
    line: string,
    cb: (err: Error | undefined, hits: [string[], string]) => void,
    query?: string
  ): Promise<void> {
    const peerIds = getPeers(this.node, {
      noBootstrapNodes: true,
    }).map(peerId => peerId.toB58String())
    const validPeerIds = query ? peerIds.filter(peerId => peerId.startsWith(query)) : peerIds

    if (!validPeerIds.length) {
      console.log(
        chalk.red(
          `\nDoesn't know any other node except apart from bootstrap node${
            this.node.bootstrapServers.length == 1 ? '' : 's'
          }!`
        )
      )
      return cb(undefined, [[''], line])
    }

    return cb(undefined, [validPeerIds.map(peerId => `send ${peerId}`), line])
  }

  async selectIntermediateNodes(rl: readline.Interface, destination: PeerId): Promise<PeerId[]> {
    let done = false
    let selected: PeerId[] = []

    // ask for node until user fills all nodes or enters an empty id
    while (!done) {
      console.log(chalk.yellow(`Please select intermediate node ${selected.length}: (leave empty to exit)`))

      const lastSelected = selected.length > 0 ? selected[selected.length - 1] : this.node.peerInfo.id
      const openChannels = await getOpenChannels(this.node, lastSelected)
      const validPeers = openChannels.map(peer => peer.toB58String())

      if (validPeers.length === 0) {
        console.log(chalk.yellow(`No peers with open channels found, you may enter a peer manually.`))
      }

      // detach prompt
      // @ts-ignore
      const oldPrompt = rl._prompt
      // @ts-ignore
      const oldCompleter = rl.completer
      const oldListeners = rl.listeners('line')
      rl.removeAllListeners('line')
      // attach new prompt
      rl.setPrompt('')
      // @ts-ignore
      rl.completer = (line: string, cb: (err: Error | undefined, hits: [string[], string]) => void) => {
        return cb(undefined, [validPeers.filter(peerId => peerId.startsWith(line)), line])
      }

      // wait for peerId to be selected
      const peerId = await new Promise<PeerId | undefined>(resolve =>
        rl.on('line', async query => {
          if (query == null || query === '\n' || query === '' || query.length == 0) {
            rl.removeAllListeners('line')
            return resolve(undefined)
          }

          let peerId: PeerId
          try {
            peerId = await checkPeerIdInput(query)
          } catch (err) {
            console.log(chalk.red(err.message))
          }

          readline.moveCursor(process.stdout, -rl.line, -1)
          readline.clearLine(process.stdout, 0)

          console.log(chalk.blue(query))

          return resolve(peerId)
        })
      )

      // no peerId selected, stop selecting nodes
      if (typeof peerId === 'undefined') {
        done = true
      }
      // check if peerId selected is destination peerId
      else if (destination.equals(peerId)) {
        console.log(chalk.yellow(`Peer selected is same as destination peer.`))
      }
      // check if peerId selected is already in the list
      else if (selected.find(p => p.equals(peerId))) {
        console.log(chalk.yellow(`Peer is already an intermediate peer.`))
      }
      // update list
      else {
        selected.push(peerId)
      }

      // we selected all peers
      if (selected.length >= MAX_HOPS - 1) {
        done = true
      }

      // reattach prompt
      rl.setPrompt(oldPrompt)
      // @ts-ignore
      rl.completer = oldCompleter
      // @ts-ignore
      oldListeners.forEach(oldListener => rl.on('line', oldListener))
    }

    return selected
  }
}
