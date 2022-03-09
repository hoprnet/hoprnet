import { styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'
import { redeemTickets } from '../fetch'

export default class RedeemTickets extends AbstractCommand {
  constructor() {
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
      log('Redeeming all tickets...')
      await redeemTickets()

      log(`Redeemed all tickets. Run 'tickets' for details`)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
