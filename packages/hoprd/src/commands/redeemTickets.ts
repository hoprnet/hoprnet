import type Hopr from '@hoprnet/hopr-core'
import { styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'

export default class RedeemTickets extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'redeemTickets'
  }

  public help() {
    return 'Redeems your tickets'
  }

  /**
   * @param query a ticket challange
   */
  public async execute(log: (str: string) => void): Promise<void> {
    try {
      const result = await this.node.redeemAllTickets()
      log(
        `Redeemed ${result.redeemed} tickets with a sum of ${styleValue(result.total.toFormattedString(), 'string')}.`
      )
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
