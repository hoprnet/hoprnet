import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions, type HoprOrNative } from '../utils/command'
import { utils as ethersUtils } from 'ethers'

export default class Balances extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'shows all balances'],
        onlyOne: [[['hoprOrNative']], 'shows shows one balance']
      },
      api,
      cache
    )
  }

  public name() {
    return 'balance'
  }

  public description() {
    return 'Displays your current HOPR and native balance'
  }

  /**
   * Prints the balance of our account.
   * @notice triggered by the CLI
   */
  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, use, type] = this.assertUsage(query) as [string | undefined, string, HoprOrNative]
    if (error) return log(error)

    const response = await this.api.getBalances()

    if (!response.ok) {
      return log(
        await this.failedApiCall(response, 'fetch balances', {
          422: (v) => v.error
        })
      )
    }

    const symbols = this.cache.getSymbols()
    const balances = await response.json()

    const hoprPrefix = `${symbols.hoprDisplay} Balance:`
    const hoprBalance = ethersUtils.formatEther(balances.hopr)
    const nativePrefix = `${symbols.nativeDisplay} Balance:`
    const nativeBalance = ethersUtils.formatEther(balances.native)

    if (use === 'onlyOne') {
      if (type === 'hopr') {
        return log(hoprBalance)
      } else {
        return log(nativeBalance)
      }
    } else {
      return log(
        toPaddedString([
          [hoprPrefix, hoprBalance],
          [nativePrefix, nativeBalance]
        ])
      )
    }
  }
}
