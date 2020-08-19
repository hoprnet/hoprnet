import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { AutoCompleteResult, CommandResponse } from './abstractCommand'
import { AbstractCommand, GlobalState } from './abstractCommand'

import chalk from 'chalk'

import type PeerId from 'peer-id'

import { checkPeerIdInput, encodeMessage, getOpenChannels, getPeers } from '../utils'
import { clearString } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '@hoprnet/hopr-core/lib/constants'

import readline from 'readline'

abstract class SendMessageBase extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() { return 'send' }
  help() { return 'sends a message to another party'}

  async _checkPeerId(id: string, settings: GlobalState): Promise<PeerId> {
    if (settings.aliases.has(id)){
      return settings.aliases.get(id)
    }
    return await checkPeerIdInput(id)
  }

  async _sendMessage(settings: GlobalState, recipient: PeerId, msg: string): Promise<void>{
    const message = settings.includeRecipient ?
      (myAddress => `${myAddress}:${msg}`)(this.node.peerInfo.id.toB58String()) :
      msg;

    try {
      return await this.node.sendMessage(encodeMessage(message), recipient)
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }

  async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    const peerIds = getPeers(this.node, {
      noBootstrapNodes: true,
    }).map((peerId) => peerId.toB58String())
    const validPeerIds = query ? peerIds.filter((peerId) => peerId.startsWith(query)) : peerIds

    if (!validPeerIds.length) {
      console.log(
        chalk.red(
          `\nDoesn't know any other node except apart from bootstrap node${
            this.node.bootstrapServers.length == 1 ? '' : 's'
          }!`
        )
      )
      return [[''], line]
    }

    return [validPeerIds.map((peerId) => `send ${peerId}`), line]
  }

}

export class SendMessageFancy extends SendMessageBase {
  constructor(public node: Hopr<HoprCoreConnector>, public rl: readline.Interface) {
    super(node)
  }

  /**
   * Encapsulates the functionality that is executed once the user decides to send a message.
   * @param query peerId string to send message to
   */
  async execute(query: string, settings: GlobalState): Promise<void> {
    if (query == null) {
      console.log(chalk.red(`Invalid arguments. Expected 'send <peerId>'. Received '${query}'`))
      return
    }

    let peerId: PeerId
    try {
      peerId = await this._checkPeerId(query, settings)
    } catch (err) {
      console.log(chalk.red(err.message))
    }

    const manualPath = process.env.MULTIHOP
      ? await (async () => {
          const manualIntermediateNodesQuestion = `Do you want to manually set the intermediate nodes? (${chalk.green(
            'y'
          )}, ${chalk.red('N')}): `
          const manualIntermediateNodesAnswer = await new Promise<string>((resolve) =>
            this.rl.question(manualIntermediateNodesQuestion, resolve)
          )

          clearString(manualIntermediateNodesQuestion + manualIntermediateNodesAnswer, this.rl)

          return (manualIntermediateNodesAnswer.toLowerCase().match(/^y(es)?$/i) || '').length >= 1
        })()
      : true

    const messageQuestion = `${chalk.yellow(`Type your message and press ENTER to send:`)}\n`
    const parsedMessage = await new Promise<string>((resolve) => this.rl.question(messageQuestion, resolve))

    const message = settings.includeRecipient ?
      (myAddress => `${myAddress}:${parsedMessage}`)(this.node.peerInfo.id.toB58String()) :
      parsedMessage;

    clearString(messageQuestion + message, this.rl)

    console.log(`Sending message to ${chalk.blue(query)} ...`)

    try {
      if (manualPath) {
        await this.node.sendMessage(encodeMessage(message), peerId, async () => {
          if (process.env.MULTIHOP) return this.selectIntermediateNodes(this.rl, peerId)
          return []
        })
      } else {
        await this.node.sendMessage(encodeMessage(message), peerId)
      }
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }

  async selectIntermediateNodes(rl: readline.Interface, destination: PeerId): Promise<PeerId[]> {
    let done = false
    let selected: PeerId[] = []

    // ask for node until user fills all nodes or enters an empty id
    while (!done) {
      console.log(chalk.yellow(`Please select intermediate node ${selected.length}: (leave empty to exit)`))

      const lastSelected = selected.length > 0 ? selected[selected.length - 1] : this.node.peerInfo.id
      const openChannels = await getOpenChannels(this.node, lastSelected)
      const validPeers = openChannels.map((peer) => peer.toB58String())

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
        return cb(undefined, [validPeers.filter((peerId) => peerId.startsWith(line)), line])
      }

      // wait for peerId to be selected
      const peerId = await new Promise<PeerId | undefined>((resolve) =>
        rl.on('line', async (query) => {
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
      else if (selected.find((p) => p.equals(peerId))) {
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
      oldListeners.forEach((oldListener) => rl.on('line', oldListener))
    }

    return selected
  }
}

export class SendMessage extends SendMessageBase {
  async execute(query: string, settings: GlobalState): Promise<CommandResponse> {
    if (query == null) {
      return `Invalid arguments. Expected 'send <peerId> <message>'. Received '${query}'`
    }

    let peerIdString: (string | undefined) = query.trim().split(' ')[0]
    let msg = query.trim().split(' ').slice(1).join(' ')

    let peerId: PeerId
    try {
      peerId = await this._checkPeerId(peerIdString, settings)
    } catch (err) {
      return err.message
    }
    this._sendMessage(settings, peerId, msg)
  }

}

