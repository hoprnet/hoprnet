import type API from '../utils/api'
import { Command } from '../utils/command'

export default class Sign extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[['string', 'message', false]], '']
      },
      api,
      extra
    )
  }

  public name() {
    return 'sign'
  }

  public description() {
    return 'Signs a message with a node’s key and the prefix "HOPR Signed Message: "'
  }

  public async execute(log, query: string): Promise<void> {
    const [error, , message] = this.assertUsage(query) as [string | undefined, string, string]
    if (error) {
      return log(error)
    }

    try {
      const response = await this.api.signMessage(message)
      const signature = await response.json()
      if (response.status === 200 || response.status === 422) {
        return log(`Signed message: ${signature.signature}`)
      } else {
        return log(`Status: ${signature.status}`)
      }
    } catch (error: any) {
      return log(error.message)
    }
  }
}
