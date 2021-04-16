import type Hopr from '@hoprnet/hopr-core'
import { moveDecimalPoint } from '@hoprnet/hopr-utils'
import { AbstractCommand, AutoCompleteResult } from './abstractCommand'
import { styleValue } from './utils'
import { Logger, Balance, NativeBalance } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.commands.withdraw')

export default class Withdraw extends AbstractCommand {
  private arguments = ['amount (ETH, HOPR)', 'currency (native, hopr)', 'recipient (blockchain address)']

  constructor(public node: Hopr) {
    super()
  }

  /**
   * Will throw if any of the arguments are incorrect.
   */
  private async checkArgs(
    query: string
  ): Promise<{
    amount: string
    weiAmount: string
    currency: 'NATIVE' | 'HOPR'
    recipient: string
  }> {
    const [err, amount, currencyRaw, recipient] = this._assertUsage(query, this.arguments)

    if (err) {
      throw new Error(err)
    }

    let currency
    if (['NATIVE', 'HOPR'].includes(currency)) {
      currency = currencyRaw.toUpperCase()
    } else {
      throw new Error(`Incorrect currency provided: '${currency}', correct options are: 'native', 'hopr'.`)
    }

    if (isNaN(Number(amount))) {
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
      recipient
    }
  }

  public name(): string {
    return 'withdraw'
  }

  public help(): string {
    return 'Withdraw native or hopr to a specified recipient'
  }

  public async autocomplete(query?: string): Promise<AutoCompleteResult> {
    return [this.arguments, query ?? '']
  }

  /**
   * Withdraws native or hopr balance.
   * @notice triggered by the CLI
   */
  public async execute(query?: string): Promise<string> {
    try {
      const { amount, weiAmount, currency, recipient } = await this.checkArgs(query ?? '')
      const symbol = currency === 'NATIVE' ? NativeBalance.SYMBOL : Balance.SYMBOL

      const receipt = await this.node.withdraw(currency, recipient, weiAmount)
      return `Withdrawing ${styleValue(amount, 'number')} ${symbol} to ${styleValue(
        recipient,
        'peerId'
      )}, receipt ${styleValue(receipt, 'hash')}.`
    } catch (err) {
      log.error('Error while withdrawing', err)
      return styleValue(err.message, 'failure')
    }
  }
}
