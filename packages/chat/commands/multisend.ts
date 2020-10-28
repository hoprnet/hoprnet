import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { clearString } from '@hoprnet/hopr-utils'
import { SendMessageBase } from './sendMessage'
import readline from 'readline'
import type PeerId from 'peer-id'
import { getPeersIdsAsString, checkPeerIdInput, styleValue } from '../utils'
import { GlobalState, AutoCompleteResult, CommandResponse } from './abstractCommand'

export class MultiSendMessage extends SendMessageBase {
  constructor(public node: Hopr<HoprCoreConnector>, public rl: readline.Interface) {
    super(node)
  }

  public name() {
    return 'multisend'
  }

  public help() {
    return 'Sends multiple messages to another party, "quit" exits'
  }

  private async checkArgs(query: string, state: GlobalState): Promise<PeerId> {
    const [err, id] = this._assertUsage(query, ['PeerId'])
    if (err) throw new Error(err)
    return await checkPeerIdInput(id, state)
  }

  private async repl(recipient: PeerId, state: GlobalState): Promise<void> {
    readline.clearLine(process.stdout, 0)
    const message = await new Promise<string>((resolve) => this.rl.question('send >', resolve))
    if (message === 'quit') {
      return
    } else {
      if (message) {
        clearString(message, this.rl)
        this.rl.pause()
        console.log(`[sending message "${message}"]`)
        await this.sendMessage(state, recipient, message)
        this.rl.resume()
      }
      await this.repl(recipient, state)
    }
  }

  public async execute(query: string, state: GlobalState): Promise<CommandResponse> {
    let peerId: PeerId

    try {
      peerId = await this.checkArgs(query, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
    await this.repl(peerId, state)
  }

  public async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeersIdsAsString(this.node, {
      noBootstrapNodes: true
    }).concat(Array.from(state.aliases.keys()))
    return this._autocompleteByFiltering(query, allIds, line)
  }
}
