import type readline from 'readline'
import { clearString } from '@hoprnet/hopr-utils'
import { CHALK_STRINGS } from './displayHelp'

export async function yesOrNoQuestion(rl: readline.Interface, message: string): Promise<boolean> {
  const question = `${message} (${CHALK_STRINGS.yes}, ${CHALK_STRINGS.no}): `
  const answer = await new Promise<string>((resolve) => rl.question(question, resolve))

  clearString(question + answer, rl)
  return (answer.toLowerCase().match(/^y(es)?$/i) || '').length >= 1
}
