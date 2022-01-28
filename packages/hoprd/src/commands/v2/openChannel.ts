import type Hopr from '@hoprnet/hopr-core'
import { styleValue } from '../utils'
import { AbstractCommand, GlobalState } from '../abstractCommand'
import { openChannel } from './logic/channel'

export class OpenChannel extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'open'
  }

  public help() {
    return 'Opens a payment channel between you and the counter party provided'
  }

  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  public async execute(log, query: string, state: GlobalState): Promise<void> {
    const [err, counterpartyStr, amountToFundStr] = this._assertUsage(query, ["counterParty's PeerId", 'amountToFund'])
    if (err) return log(styleValue(err, 'failure'))
    await openChannel({ amountToFundStr, counterpartyPeerId: counterpartyStr, node: this.node, state, log })
  }
}
