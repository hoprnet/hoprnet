import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { signMessage } from '../fetch'

export default class Sign extends AbstractCommand {
  constructor() {
    super()
  }

  public name() {
    return 'sign'
  }

  public help() {
    return 'Signs a message with a node’s key and the prefix "HOPR Signed Message: "'
  }

  public async execute(log, query: string): Promise<void> {
    if (!query) {
      return log(`Invalid arguments. Expected 'sign <message>'. Received '${query}'`)
    }

    try {
      const signature = await signMessage(query).then(res => res.json()).then(res => res.signature)

      return log(`Signed message: ${signature}`)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
