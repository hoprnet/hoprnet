import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { getBalances } from '../fetch'

export default class PrintBalance extends AbstractCommand {
  constructor() {
    super()
  }

  public name() {
    return 'balance'
  }

  public help() {
    return 'Displays your current HOPR and native balance'
  }

  /**
   * Prints the balance of our account.
   * @notice triggered by the CLI
   */
  public async execute(log): Promise<void> {
    const balances = await getBalances()

    const hoprPrefix = 'HOPR Balance:'
    // TODO: toFormattedString()
    const hoprBalance = balances.hopr

    const nativePrefix = 'ETH Balance:'
    const nativeBalance = balances.native

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    // TODO: use 'NativeBalance' and 'Balance' to display currencies
    return log(
      [
        `${hoprPrefix.padEnd(prefixLength, ' ')}${styleValue(hoprBalance, 'number')}`,
        `${nativePrefix.padEnd(prefixLength, ' ')}${styleValue(nativeBalance, 'number')}`
      ].join('\n')
    )
  }
}
