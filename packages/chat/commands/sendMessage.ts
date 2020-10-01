import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { AutoCompleteResult, CommandResponse } from './abstractCommand'
import { AbstractCommand, GlobalState } from './abstractCommand'

import chalk from 'chalk'

import type PeerId from 'peer-id'

import { checkPeerIdInput, encodeMessage, getOpenChannels, getPeersIdsAsString, yesOrNoQuestion } from '../utils'
import { clearString } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '@hoprnet/hopr-core/lib/constants'

import readline from 'readline'

export abstract class SendMessageBase extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'send'
  }

  help() {
    return 'sends a message to another party'
  }

  // Throws if peerid is invalid
  async _checkPeerId(id: string, settings: GlobalState): Promise<PeerId> {
    if (settings.aliases.has(id)) {
      return settings.aliases.get(id)!
    }
    return await checkPeerIdInput(id)
  }

  async _sendMessage(settings: GlobalState, recipient: PeerId, msg: string): Promise<void> {
    const message = settings.includeRecipient
      ? ((myAddress) => `${myAddress}:${msg}`)(this.node.peerInfo.id.toB58String())
      : msg

    try {
      return await this.node.sendMessage(
        encodeMessage(message),
        recipient,
        async () => [] // MULTIHOP not supported
      )
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }

  async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeersIdsAsString(this.node, {
      noBootstrapNodes: true,
    }).concat(Array.from(state.aliases.keys()))
    return this._autocompleteByFiltering(query, allIds, line)
  }
}

export class SendMessage extends SendMessageBase {
  async execute(query: string, settings: GlobalState): Promise<CommandResponse> {
    const [err, peerIdString, msg] = this._assertUsage(query, ['PeerId', 'Message'], /(\w+)\s(.*)/)
    if (err) return err

    let peerId: PeerId
    try {
      peerId = await this._checkPeerId(peerIdString, settings)
    } catch (err) {
      return err.message
    }
    this._sendMessage(settings, peerId, msg)
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
  async execute(query: string, state: GlobalState): Promise<string | void> {
    const [err, peerIdString] = this._assertUsage(query, ['PeerId'])
    if (err) return err
    let peerId: PeerId
    try {
      peerId = await this._checkPeerId(peerIdString, state)
    } catch (err) {
      console.log(chalk.red(err.message))
      return
    }

    const messageQuestion = `${chalk.yellow(`Type your message and press ENTER to send:`)}\n`
    const parsedMessage = await new Promise<string>((resolve) => this.rl.question(messageQuestion, resolve))

    const message = state.includeRecipient
      ? ((myAddress) => `${myAddress}:${parsedMessage}`)(this.node.peerInfo.id.toB58String())
      : parsedMessage

    clearString(messageQuestion + message, this.rl)
    console.log(`Sending message to ${chalk.blue(query)} ...`)

    try {
      // use manual path
      if (state.routing === 'manual') {
        await this.node.sendMessage(encodeMessage(message), peerId, async () => {
          return this.selectIntermediateNodes(this.rl, peerId)
        })
      }
      // use random path
      else if (state.routing === 'auto') {
        // @TODO: use path finder
        await this.node.sendMessage(encodeMessage(message), peerId)
      }
      // 0 hops
      else {
        await this.node.sendMessage(encodeMessage(message), peerId)
      }
    } catch (err) {
      return chalk.red(err.message)
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
            return
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
      // @TODO: handle self
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
