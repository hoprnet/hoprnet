import type API from '../utils/api'
import { utils as ethersUtils } from 'ethers'
import { Command } from '../utils/command'

export default class Withdraw extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [
          [
            ['number', 'amount to withdraw', false],
            ['hoprOrNative', 'withdraw HOPR or NATIVE', false],
            ['nativeAddress', 'recipient', false]
          ],
          'withdraw'
        ]
      },
      api,
      extra
    )
  }

  public name(): string {
    return 'withdraw'
  }

  public description(): string {
    return 'Withdraw native or hopr to a specified recipient'
  }

  /**
   * Withdraws native or hopr balance.
   */
  public async execute(log, query: string): Promise<void> {
    const [error, , amount, currency, recipient] = this.assertUsage(query) as [
      string | undefined,
      string,
      number,
      string,
      string
    ]
    if (error) return log(error)

    const amountWei = ethersUtils.parseEther(String(amount))

    try {
      const response = await this.api.withdraw(amountWei.toString(), currency, recipient)

      if (response.status === 200) {
        const receipt = response.json().then((res) => res.receipt)
        return log(`Withdrawing ${amount} ${currency} to ${recipient}, receipt ${receipt}.`)
      } else {
        return log(`Unable to withdraw. ${response.status}`)
      }
    } catch (err) {
      return log(`Unexpected error: ${err.message}`)
    }
  }
}
