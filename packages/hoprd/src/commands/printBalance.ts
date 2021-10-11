import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'
import { styleValue } from './utils/index.js'

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
    const hoprBalance = (await this.node.getBalance()).toFormattedString()

    const nativePrefix = 'ETH Balance:'
    const nativeBalance = (await this.node.getNativeBalance()).toFormattedString()

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
