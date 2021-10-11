import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import chalk from 'chalk'
import { checkPeerIdInput, styleValue } from './utils/index.js'
import { AbstractCommand, GlobalState } from './abstractCommand.js'

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

    let counterparty: PeerId
    try {
      counterparty = await checkPeerIdInput(counterpartyStr, state)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    const amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
    const myAvailableTokens = await this.node.getBalance()
    if (amountToFund.lten(0)) {
      return log(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      return log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    }

    log('Opening channel...')

    try {
      const { channelId } = await this.node.openChannel(counterparty, amountToFund)
      return log(`${chalk.green(`Successfully opened channel`)} ${styleValue(channelId.toHex(), 'hash')}`)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
