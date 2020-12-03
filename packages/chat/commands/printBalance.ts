import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from '../utils'

export default class PrintBalance extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
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
  public async execute(): Promise<string> {
    const hoprPrefix = 'HOPR Balance:'
    const hoprBalance = (await this.node.getBalance()).toFormattedString()

    const nativePrefix = 'BNB Balance:'
    const nativeBalance = (await this.node.getNativeBalance()).toFormattedString()

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    // TODO: use 'NativeBalance' and 'Balance' to display currencies
    return [
      `${hoprPrefix.padEnd(prefixLength, ' ')}${styleValue(hoprBalance, 'number')}`,
      `${nativePrefix.padEnd(prefixLength, ' ')}${styleValue(nativeBalance, 'number')}`
    ].join('\n')
  }
}
