import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import HoprFetcher from '../fetch'

export default class Sign extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
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
      const response = await this.hoprFetcher.signMessage(query)
      const signature = await response.json()
      if (response.status === 200 || response.status === 422) {
        return log(`Signed message: ${signature.signature}`)
      } else {
        return log(`Status: ${signature.status}`)
      }
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
