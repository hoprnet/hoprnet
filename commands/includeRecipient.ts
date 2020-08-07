import type AbstractCommand from './abstractCommand'
import chalk from 'chalk'
import readline from 'readline'
import { clearString } from '@hoprnet/hopr-utils'
import { settings } from '../utils'

export default class IncludeRecipient implements AbstractCommand {
  async execute(rl: readline.Interface): Promise<void> {
    const question = `Are you sure you want to include your address in your messages? (${chalk.green('y')}, ${chalk.red('N')}): `
    const answer = await new Promise<string>((resolve) => rl.question(question, resolve))

    // Bitwise a Regex Expression to force [] => true and null => false
    settings.includeRecipient = !!answer.match(/^y(es)?$/i)

    clearString(question, rl)
    console.log(`You have set your “includeRecipient” settings to ${ settings.includeRecipient ? chalk.green('yes') : chalk.red('no') }`)
  }

  complete() {}
}
