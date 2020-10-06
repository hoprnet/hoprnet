import type readline from 'readline'
import chalk from 'chalk'
import { clearString } from '@hoprnet/hopr-utils'

const yes = chalk.green('y')
const no = chalk.red('N')

export async function yesOrNoQuestion(rl: readline.Interface, message: string) {
  const question = `${message} (${yes}, ${no}): `
  const answer = await new Promise<string>((resolve) => rl.question(question, resolve))

  clearString(question + answer, rl)
  return (answer.toLowerCase().match(/^y(es)?$/i) || '').length >= 1
}
