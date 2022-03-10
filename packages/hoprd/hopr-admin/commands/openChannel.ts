import type PeerId from 'peer-id'
import chalk from 'chalk'
import { checkPeerIdInput, styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'
import { getBalances, setChannels } from '../fetch'
import { BalanceDecimals, hoprToWei } from './utils/util'
import {  moveDecimalPoint } from './utils/moveDecimal'
import BN from 'bn.js'

export class OpenChannel extends AbstractCommand {
  constructor() {
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
  public async execute(log, query: string): Promise<void> {
    const [err, counterpartyStr, amountToFundStr] = this._assertUsage(query, ["counterParty's PeerId", 'amountToFund'])
    if (err) return log(styleValue(err, 'failure'))

    let counterparty: PeerId
    try {
      counterparty = checkPeerIdInput(counterpartyStr)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    // TODO: Add hoprToWei function ??
    const amountToFund = new BN(moveDecimalPoint(amountToFundStr, BalanceDecimals.Balance))

    const myAvailableTokens = await getBalances()
    // TODO: if (amountToFund.lten(0))
    if (amountToFund) {
      return log(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      return log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    }

    log('Opening channel...')
    //
    // try {
    //   // const { peerId, amount } = await setChannels(counterparty.id, 1000);
    //   const { channelId } = await this.node.openChannel(counterparty, amountToFund)
    //   return log(`${chalk.green(`Successfully opened channel`)} ${styleValue(channelId.toHex(), 'hash')}`)
    // } catch (err) {
    //   return log(styleValue(err.message, 'failure'))
    // }
  }
}
