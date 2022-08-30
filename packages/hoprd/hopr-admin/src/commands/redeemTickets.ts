import type API from '../utils/api'
import { Command, type CacheFunctions } from '../utils/command'

export default class RedeemTickets extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'redeem all tickets']
      },
      api,
      cache
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
    if (!response.ok) return log(this.failedCommand('redeem tickets'))
    log(`Redeemed all tickets. Run 'tickets' for details`)
  }
}
