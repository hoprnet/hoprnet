import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type { AutoCompleteResult, CommandResponse } from './abstractCommand'
import type PeerId from 'peer-id'
import { clearString } from '@hoprnet/hopr-utils'
import { MAX_HOPS } from '@hoprnet/hopr-core/lib/constants'
import readline from 'readline'
import chalk from 'chalk'
import { checkPeerIdInput, encodeMessage, getOpenChannels, getPeerIdsAndAliases, styleValue } from '../utils'
import { AbstractCommand, GlobalState } from './abstractCommand'

export abstract class SendMessageBase extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'send'
  }

  public help() {
    return 'Sends a message to another party'
  }

  protected async sendMessage(
    state: GlobalState,
    recipient: PeerId,
    msg: string,
    getIntermediateNodes?: () => Promise<PeerId[]>
  ): Promise<string | void> {
    const message = state.includeRecipient
      ? ((myAddress) => `${myAddress}:${msg}`)(this.node.peerInfo.id.toB58String())
      : msg

    console.log(`Sending message to ${styleValue(recipient.toB58String(), 'peerId')} ...`)

    try {
      let m = encodeMessage(message)
      /*if (state.routing === 'auto') {
        // use random path
        return await this.node.sendMessage(m, recipient)
      } else
      */
      if (state.routing === 'direct') {
        // 0 hops
        return await this.node.sendMessage(m, recipient, async () => [])
      } else {
        let path = await Promise.all(state.routing.split(',').map(async (x) => await checkPeerIdInput(x)))
        return await this.node.sendMessage(m, recipient, () => Promise.resolve(path))
      }
    } catch (err) {
      return styleValue('Could not send message.', 'failure')
    }
  }

  public async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeerIdsAndAliases(this.node, state, {
      noBootstrapNodes: true,
      returnAlias: true,
      mustBeOnline: true
    })
    return this._autocompleteByFiltering(query, allIds, line)
  }
}

export class SendMessage extends SendMessageBase {
  public async execute(query: string, state: GlobalState): Promise<CommandResponse> {
    const [err, peerIdString, msg] = this._assertUsage(query, ['PeerId', 'Message'], /(\w+)\s(.*)/)
    if (err) return err

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(peerIdString, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    return this.sendMessage(state, peerId, msg)
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
  public async execute(query: string, state: GlobalState): Promise<string | void> {
    const [err, peerIdString] = this._assertUsage(query, ['PeerId'])
    if (err) return err

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(peerIdString, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    const messageQuestion = styleValue(`Type your message and press ENTER to send:`, 'highlight') + '\n'
    const message = await new Promise<string>((resolve) => this.rl.question(messageQuestion, resolve))
    clearString(messageQuestion + message, this.rl)

    try {
      // use intermediate nodes
      if (state.routing === 'manual') {
        return await this.sendMessage(state, peerId, message, async () => {
          return this.selectIntermediateNodes(this.rl, peerId)
        })
      }
      // 0 HOP
      else {
        return await this.sendMessage(state, peerId, message, async () => [])
      }
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }

  public async selectIntermediateNodes(rl: readline.Interface, destination: PeerId): Promise<PeerId[]> {
    let done = false
    let selected: PeerId[] = []

    // ask for node until user fills all nodes or enters an empty id
    while (!done) {
      console.log(
        styleValue(`Please select intermediate node ${styleValue(selected.length)}: (leave empty to exit)`, 'highlight')
      )

      const lastSelected = selected.length > 0 ? selected[selected.length - 1] : this.node.peerInfo.id
      const openChannels = await getOpenChannels(this.node, lastSelected)
      const validPeers = openChannels.map((peer) => peer.toB58String())

      if (validPeers.length === 0) {
        console.log(styleValue(`No peers with open channels found, you may enter a peer manually.`, 'highlight'))
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
            console.log(styleValue(err.message, 'failure'))
            return
          }

          readline.moveCursor(process.stdout, -rl.line, -1)
          readline.clearLine(process.stdout, 0)

          return resolve(peerId)
        })
      )
      rl.removeAllListeners('line')

      // no peerId selected, stop selecting nodes
      if (typeof peerId === 'undefined') {
        done = true
      }
      // @TODO: handle self
      // check if peerId selected is destination peerId
      else if (destination.equals(peerId)) {
        console.log(styleValue(`Peer selected is same as destination peer.`, 'failure'))
      }
      // check if peerId selected is already in the list
      else if (selected.find((p) => p.equals(peerId))) {
        console.log(styleValue(`Peer is already an intermediate peer.`, 'failure'))
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
