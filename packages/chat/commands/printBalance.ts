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
    const { paymentChannels } = this.node
    const { Balance, NativeBalance } = paymentChannels.types

    const hoprPrefix = 'HOPR Balance:'
    const hoprBalance = await paymentChannels.account.balance.then((b) => {
      return moveDecimalPoint(b.toString(), Balance.DECIMALS * -1)
    })

    // @TODO: use 'NativeBalance' and 'Balance' to display currencies
    const nativePrefix = 'Matic Balance:'
    const nativeBalance = await paymentChannels.account.nativeBalance.then((b) => {
      return moveDecimalPoint(b.toString(), NativeBalance.DECIMALS * -1)
    })

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    // TODO: use 'NativeBalance' and 'Balance' to display currencies
    return [
      `${hoprPrefix.padEnd(prefixLength, ' ')}${styleValue(hoprBalance, 'number')} HOPR`,
      `${nativePrefix.padEnd(prefixLength, ' ')}${styleValue(nativeBalance, 'number')} Matic`
    ].join('\n')
  }
}
