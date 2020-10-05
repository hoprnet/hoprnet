import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Currencies } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { AbstractCommand, AutoCompleteResult } from './abstractCommand'

const _arguments = ['recipient (blockchain address)', 'currency (native, hopr)', 'amount (ETH, HOPR)']

export default class Withdraw extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name(): string {
    return 'withdraw'
  }

  help(): string {
    return 'withdraw native or hopr to a specified recipient'
  }

  async autocomplete(query?: string): Promise<AutoCompleteResult> {
    return [_arguments, query ?? '']
  }

  /**
   * Will throw if any of the arguments are incorrect.
   */
  private async checkArgs(
    query: string
  ): Promise<{
    amount: string
    weiAmount: string
    currency: Currencies
    recipient: string
  }> {
    const { NativeBalance, Balance } = this.node.paymentChannels.types

    const [err, amount, currencyRaw, recipient] = this._assertUsage(query, [
      'amount (ETH, HOPR)',
      'currency (native, hopr)',
      'recipient (blockchain address)',
    ])

    if (err) {
      throw new Error(err)
    }

    const currency = currencyRaw.toUpperCase() as Currencies

    if (!['NATIVE', 'HOPR'].includes(currency)) {
      throw new Error(`Incorrect currency provided: '${currency}', correct options are: 'native', 'hopr'.`)
    } else if (isNaN(Number(amount))) {
      throw new Error(`Incorrect amount provided: '${amount}'.`)
    }

    // @TODO: validate recipient address

    const weiAmount =
      currency === 'NATIVE'
        ? moveDecimalPoint(amount, NativeBalance.DECIMALS)
        : moveDecimalPoint(amount, Balance.DECIMALS)

    return {
      amount,
      weiAmount,
      currency,
      recipient,
    }
  }

  /**
   * Withdraws native or hopr balance.
   * @notice triggered by the CLI
   */
  async execute(query?: string): Promise<string> {
    try {
      const { amount, weiAmount, currency, recipient } = await this.checkArgs(query ?? '')
      const { paymentChannels } = this.node
      const { NativeBalance, Balance } = paymentChannels.types
      const symbol = currency === 'NATIVE' ? NativeBalance.SYMBOL : Balance.SYMBOL

      const receipt = await paymentChannels.withdraw(currency, recipient, weiAmount)
      return `Withdrawing ${chalk.blue(amount)} ${symbol} to ${chalk.green(recipient)}, receipt ${chalk.yellow(
        receipt
      )}.`
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
