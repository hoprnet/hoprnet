import type API from '../utils/api'
import { Command } from '../utils/command'

export default class RedeemTickets extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[], 'redeem all tickets']
      },
      api,
      extra
    )
  }

  public name() {
    return 'redeemTickets'
  }

  public description() {
    return 'Redeems all your tickets'
  }

  /**
   * @param query a ticket challange
   */
  public async execute(log: (str: string) => void): Promise<void> {
    try {
      log('Redeeming all tickets...')
      await this.api.redeemTickets()

      log(`Redeemed all tickets. Run 'tickets' for details`)
    } catch (error) {
      return log(error.message)
    }
  }
}
