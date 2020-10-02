import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import chalk from 'chalk'

export default class PrintBalance extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'balance'
  }

  help() {
    return 'shows our current hopr and native balance'
  }

  /**
   * Prints the balance of our account.
   * @notice triggered by the CLI
   */
  async execute(): Promise<string> {
    const { paymentChannels } = this.node
    const { Balance, NativeBalance } = paymentChannels.types

    const hoprPrefix = 'HOPR Balance:'
    const hoprBalance = await paymentChannels.account.balance.then((b) => {
      return moveDecimalPoint(b.toString(), Balance.DECIMALS * -1)
    })

    // @TODO: use 'NativeBalance' and 'Balance' to display currencies
    const nativePrefix = 'xDAI Balance:'
    const nativeBalance = await paymentChannels.account.nativeBalance.then((b) => {
      return moveDecimalPoint(b.toString(), NativeBalance.DECIMALS * -1)
    })

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    // TODO: use 'NativeBalance' and 'Balance' to display currencies
    return [
      `${hoprPrefix.padEnd(prefixLength, ' ')}${chalk.blue(hoprBalance)} xHOPR`,
      `${nativePrefix.padEnd(prefixLength, ' ')}${chalk.blue(nativeBalance)} xDAI`,
    ].join('\n')
  }
}
