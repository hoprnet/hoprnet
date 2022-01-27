import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from '../abstractCommand'
import { styleValue } from '../utils'
import { CommandE } from '.'
import { withdraw } from './logic/withdraw'

export default class Withdraw extends AbstractCommand {
  private arguments = ['amount (ETH, HOPR)', 'currency (native, hopr)', 'recipient (blockchain address)']

  constructor(public node: Hopr) {
    super()
  }

  public name(): string {
    return CommandE.WITHDRAW
  }

  public help(): string {
    return 'Withdraw native or hopr to a specified recipient'
  }

  /**
   * Withdraws native or hopr balance.
   * @notice triggered by the CLI
   */
  public async execute(log, query: string): Promise<void> {
    try {
      const [err, rawAmount, rawCurrency, rawRecipient] = this._assertUsage(query, this.arguments)

      if (err) {
        throw new Error(err)
      }

      await withdraw({ rawCurrency, rawRecipient, rawAmount, node: this.node })
    } catch (err) {
      log(styleValue(err.message, 'failure'))
    }
  }
}
