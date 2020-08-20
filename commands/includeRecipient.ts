import { AbstractCommand, GlobalState } from './abstractCommand'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'
import readline from 'readline'
import { clearString } from '@hoprnet/hopr-utils'

export class IncludeRecipient extends AbstractCommand {
  name() { return 'includeRecipient' }
  help() { return 'preprends your address to all messages' }

  async execute(query: string, settings: GlobalState): Promise<string | void> {
    if (!query.match(/true|false/i)){
      return "includeRecipient takes an argument 'true' or 'false'"
    }
    settings.includeRecipient = !!query.match(/true/i)
  }
}

export class IncludeRecipientFancy extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>, public rl: readline.Interface) {
    super()
  }

  name() { return 'includeRecipient' }
  help() { return 'preprends your address to all messages' }

  async execute(query: string, settings: GlobalState): Promise<void> {
    const question = `Are you sure you want to include your address in your messages? (${chalk.green('y')}, ${chalk.red('N')}): `
    const answer = await new Promise<string>((resolve) => this.rl.question(question, resolve))

    // Bitwise a Regex Expression to force [] => true and null => false
    settings.includeRecipient = !!answer.match(/^y(es)?$/i)

    clearString(question, this.rl)
    console.log(`You have set your “includeRecipient” settings to ${ settings.includeRecipient ? chalk.green('yes') : chalk.red('no') }`)
  }
}


