import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions } from '../utils/command'
import { utils } from 'ethers'

export default class Tickets extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'shows all tickets']
      },
      api,
      cache
    )
  }

  public name() {
    return 'tickets'
  }

  public description() {
    return 'Displays information about your redeemed and unredeemed tickets'
  }

  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    const response = await this.api.getTicketStats()
    if (!response.ok) {
      return log(this.failedCommand('get ticket statistics'))
    } else {
      const stats = await response.json()
      const symbols = this.cache.getSymbols()
      return log(
        toPaddedString([
          ['Tickets:', ''],
          ['- Pending:', stats.pending],
          ['- Unredeemed:', stats.unredeemed],
          ['- Unredeemed Value:', `${utils.formatEther(stats.unredeemedValue)} ${symbols.hopr}`],
          ['- Redeemed:', stats.redeemed],
          ['- Redeemed Value:', `${utils.formatEther(stats.redeemedValue)} ${symbols.hopr}`],
          ['- Losing Tickets:', stats.losingTickets],
          ['- Win Proportion:', stats.winProportion * 100],
          ['- Neglected:', stats.neglected],
          ['- Rejected:', stats.rejected],
          ['- Rejected Value:', `${utils.formatEther(stats.rejectedValue)} ${symbols.hopr}`]
        ])
      )
    }
  }
}
