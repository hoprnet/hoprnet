import { styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'
import HoprFetcher from '../fetch'

export default class RedeemTickets extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
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
      await this.hoprFetcher.redeemTickets()

      log(`Redeemed all tickets. Run 'tickets' for details`)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
