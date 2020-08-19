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
    return 'print'
  }

  help():string {
    return 'shows our current balance'
  }

  /**
   * Prints the balance of our account.
   * @notice triggered by the CLI
   */
  async execute(): Promise<void> {
    const { paymentChannels } = this.node
    const { Balance, NativeBalance } = paymentChannels.types

    const balance = await paymentChannels.account.balance.then((b) => {
      return moveDecimalPoint(b.toString(), Balance.DECIMALS * -1)
    })
    const nativeBalance = await paymentChannels.account.nativeBalance.then((b) => {
      return moveDecimalPoint(b.toString(), NativeBalance.DECIMALS * -1)
    })

    console.log(
      [
        `Account Balance: ${chalk.magenta(balance)} ${Balance.SYMBOL}`,
        `Account Native Balance: ${chalk.magenta(nativeBalance)} ${NativeBalance.SYMBOL}`,
      ].join('\n')
    )
  }
}
