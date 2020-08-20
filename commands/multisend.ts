import { SendMessageBase } from './sendMessage'
import { AbstractCommand, GlobalState } from './abstractCommand'
import type { AutoCompleteResult, CommandResponse } from './abstractCommand'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import readline from 'readline'
import chalk from 'chalk'
import type PeerId from 'peer-id'

export class MultiSendMessage extends SendMessageBase {
  constructor(public node: Hopr<HoprCoreConnector>, public rl: readline.Interface) {
    super(node)
  }

  name() { return 'multisend' }
  help() { return 'sends multiple messages to another party, "quit" exits.'}

  private async checkArgs(query: string, settings: GlobalState): Promise<PeerId> {
    if (query == null) {
      throw new Error(`Invalid arguments. Usage: 'multisend <peerId>'.`)
    }
    return await this._checkPeerId(query, settings)
  }

  private async repl(recipient: PeerId, settings: GlobalState): Promise<void>{
    const message = await new Promise<string>(resolve => this.rl.question('>', resolve))
    if (message === 'quit') {
      return;
    } else {
      await this._sendMessage(settings, recipient, message)
      await this.repl(recipient, settings);
    }
  }

  async execute(query: string, settings: GlobalState): Promise<CommandResponse> {
    let peerId: PeerId;

    try {
      peerId = await this.checkArgs(query, settings)
    } catch (err) {
      return chalk.red(err.message)
    }
    await this.repl(peerId, settings)
  }

}
