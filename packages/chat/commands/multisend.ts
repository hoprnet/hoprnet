import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { clearString } from '@hoprnet/hopr-utils'
import { SendMessageBase } from './sendMessage'
import readline from 'readline'
import chalk from 'chalk'
import type PeerId from 'peer-id'
import { getPeersIdsAsString } from '../utils'
import { GlobalState, AutoCompleteResult, CommandResponse } from './abstractCommand'

export class MultiSendMessage extends SendMessageBase {
  constructor(public node: Hopr<HoprCoreConnector>, public rl: readline.Interface) {
    super(node)
  }

  public name() {
    return 'multisend'
  }

  public help() {
    return 'sends multiple messages to another party, "quit" exits.'
  }

  private async checkArgs(query: string, settings: GlobalState): Promise<PeerId> {
    const [err, id] = this._assertUsage(query, ['PeerId'])
    if (err) throw new Error(err)
    return await this._checkPeerId(id, settings)
  }

  private async repl(recipient: PeerId, settings: GlobalState): Promise<void> {
    readline.clearLine(process.stdout, 0)
    const message = await new Promise<string>((resolve) => this.rl.question('send >', resolve))
    if (message === 'quit') {
      return
    } else {
      if (message) {
        clearString(message, this.rl)
        this.rl.pause()
        console.log(`[sending message "${message}"]`)
        await this._sendMessage(settings, recipient, message)
        this.rl.resume()
      }
      await this.repl(recipient, settings)
    }
  }

  public async execute(query: string, settings: GlobalState): Promise<CommandResponse> {
    let peerId: PeerId

    try {
      peerId = await this.checkArgs(query, settings)
    } catch (err) {
      return chalk.red(err.message)
    }
    await this.repl(peerId, settings)
  }

  public async autocomplete(query: string, line: string, state: GlobalState): Promise<AutoCompleteResult> {
    const allIds = getPeersIdsAsString(this.node, {
      noBootstrapNodes: true,
    }).concat(Array.from(state.aliases.keys()))
    return this._autocompleteByFiltering(query, allIds, line)
  }
}
