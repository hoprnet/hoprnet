import type API from '../utils/api'
import { Command, type CacheFunctions } from '../utils/command'

export default class Sign extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['string', 'message']], 'Signs a message']
      },
      api,
      cache
    )
  }

  public name() {
    return 'sign'
  }

  public description() {
    return 'Signs a message with a node’s key and the prefix "HOPR Signed Message: "'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , message] = this.assertUsage(query) as [string | undefined, string, string]
    if (error) {
      return log(error)
    }

    const response = await this.api.signMessage(message)
    if (!response.ok) {
      return log(this.failedCommand('sign message'))
    } else {
      return log(`Signed message: ${(await response.json()).signature}`)
    }
  }
}
