import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type AbstractCommand from './abstractCommand'

import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import chalk from 'chalk'

export default class PrintBalance implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * Prints the balance of our account.
   * @notice triggered by the CLI
   */
  async execute(): Promise<void> {
    const { paymentChannels } = this.node
    const { Balance, NativeBalance } = paymentChannels.types

    const balance = await paymentChannels.accountBalance.then(b => {
      return moveDecimalPoint(b.toString(), Balance.DECIMALS * -1)
    })
    const nativeBalance = await paymentChannels.accountNativeBalance.then(b => {
      return moveDecimalPoint(b.toString(), NativeBalance.DECIMALS * -1)
    })

    console.log(
      [
        `Account Balance: ${chalk.magenta(balance)} ${Balance.SYMBOL}`,
        `Account Native Balance: ${chalk.magenta(nativeBalance)} ${NativeBalance.SYMBOL}`,
      ].join('\n')
    )
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
    cb(undefined, [[''], line])
  }
}
