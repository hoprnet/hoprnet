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
  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    log('Redeeming all tickets...')
    const response = await this.api.redeemTickets()
    if (!response.ok) return log(this.invalidResponse('redeem tickets'))
    log(`Redeemed all tickets. Run 'tickets' for details`)
  }
}
