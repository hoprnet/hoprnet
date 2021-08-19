import type Hopr from '@hoprnet/hopr-core'
import { u8aToHex } from '@hoprnet/hopr-utils'

import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'

export default class Sign extends AbstractCommand {
  constructor(public node: Hopr) {
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
      const signature = await this.node.signMessage(new TextEncoder().encode(query))
      return log(`Signed message: ${u8aToHex(signature)}`)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
