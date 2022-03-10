import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { getTicketStats } from '../fetch'
import { toFormattedString } from './utils/formatting'
import { BalanceSymbols } from './utils/util'

export default class Tickets extends AbstractCommand {
  constructor() {
    super()
  }

  public name() {
    return 'tickets'
  }

  public help() {
    return 'Displays information about your redeemed and unredeemed tickets'
  }

  public async execute(log): Promise<void> {
    log('finding information about tickets...')
    try {
      const stats = await getTicketStats()

      log(`()
Tickets:
- Pending:          ${stats.pending}
- Unredeemed:       ${stats.unredeemed}
- Unredeemed Value: ${toFormattedString(stats.unredeemedValue, BalanceSymbols.Balance)}
- Redeemed:         ${stats.redeemed}
- Redeemed Value:   ${toFormattedString(stats.redeemedValue, BalanceSymbols.Balance)}
- Losing Tickets:   ${stats.losingTickets}
- Win Proportion:   ${stats.winProportion * 100}% 
- Neglected:        ${stats.neglected} 
- Rejected:         ${stats.rejected}
- Rejected Value:   ${toFormattedString(stats.rejectedValue, BalanceSymbols.Balance)}
          `)
    } catch (err) {
      log(styleValue(err.message, 'failure'))
    }
  }
}
