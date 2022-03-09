import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { getTicketStats } from '../fetch'

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
- Unredeemed Value: ${stats.unredeemedValue}
- Redeemed:         ${stats.redeemed}
- Redeemed Value:   ${stats.redeemedValue}
- Losing Tickets:   ${stats.losingTickets}
- Win Proportion:   ${stats.winProportion * 100}% 
- Neglected:        ${stats.neglected} 
- Rejected:         ${stats.rejected}
- Rejected Value:   ${stats.rejectedValue}
          `)
    } catch (err) {
      log(styleValue(err.message, 'failure'))
    }
  }
}
