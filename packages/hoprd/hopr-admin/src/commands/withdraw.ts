import type API from '../utils/api'
import { utils as ethersUtils } from 'ethers'
import { Command, type CacheFunctions, type HoprOrNative } from '../utils/command'

export default class Withdraw extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['number', 'amount'], ['hoprOrNative'], ['nativeAddress']], `withdraw 'hopr' or 'native' tokens`]
      },
      api,
      cache
    )
  }

  public name(): string {
    return 'withdraw'
  }

  public description(): string {
    return 'Withdraw hopr or native tokens to a given native address'
  }

  /**
   * Withdraws hopr or native tokens.
   */
  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , amount, currency, recipient] = this.assertUsage(query) as [
      string | undefined,
      string,
      number,
      HoprOrNative,
      string
    ]
    if (error) return log(error)

    const amountWei = ethersUtils.parseEther(String(amount))

    log(`Withdrawing ${amount} ${currency} to "${recipient}"..`)
    const response = await this.api.withdraw(amountWei.toString(), currency, recipient)

    if (!response.ok) {
      return log(
        await this.failedApiCall(response, `withdraw ${amount} ${currency}`, {
          400: (v) => `one or more invalid inputs ${v.status}`,
          422: (v) => v.status
        })
      )
    }

    const receipt = await response.json().then((res) => res.receipt)
    return log(`Withdrawed ${amount} ${currency} to "${recipient}", receipt ${receipt}.`)
  }
}
