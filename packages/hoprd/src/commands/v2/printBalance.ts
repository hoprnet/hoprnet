import type Hopr from '@hoprnet/hopr-core'
import { isError } from '.'
import { AbstractCommand } from '../abstractCommand'
import { styleValue } from '../utils'
import { getBalances } from './logic/balance'

export default class PrintBalance extends AbstractCommand {
  constructor(public node: Hopr) {
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
    const hoprPrefix = 'HOPR Balance:'
    const nativePrefix = 'ETH Balance:'

    const balances = await getBalances(this.node)
    if (isError(balances)) {
      return log('Error getting balances')
    }

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    // TODO: use 'NativeBalance' and 'Balance' to display currencies
    return log(
      [
        `${hoprPrefix.padEnd(prefixLength, ' ')}${styleValue(balances.hopr, 'number')}`,
        `${nativePrefix.padEnd(prefixLength, ' ')}${styleValue(balances.native, 'number')}`
      ].join('\n')
    )
  }
}
