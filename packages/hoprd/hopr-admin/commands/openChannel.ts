import type PeerId from 'peer-id'
import chalk from 'chalk'
import { styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'
import { BalanceDecimals } from './utils/types'
import { moveDecimalPoint } from './utils/moveDecimal'
import BN from 'bn.js'
import HoprFetcher from '../fetch'

export class OpenChannel extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
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
      counterparty = await this.checkPeerIdInput(counterpartyStr)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    const amountToFund = new BN(moveDecimalPoint(amountToFundStr, BalanceDecimals.Balance))
    const myAvailableTokens = await this.hoprFetcher.getBalances().then((d) => new BN(d.hopr))

    if (amountToFund.lten(0)) {
      return log(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens)) {
      return log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toString(10)}`)
    }

    log('Opening channel...')

    try {
      const response = await this.hoprFetcher.setChannels(counterparty.toB58String(), amountToFund.toString())
      if (response.status == 201) {
        const channelId = response.json().then((res) => res.channelId)
        // TODO: channelId.toHex()
        return log(`${chalk.green(`Successfully opened channel`)} ${styleValue(channelId, 'hash')}`)
      } else {
        const status = response.json().then((res) => res.status)
        return log(status)
      }
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
