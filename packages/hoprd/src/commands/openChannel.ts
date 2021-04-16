import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import chalk from 'chalk'
import { checkPeerIdInput, styleValue } from './utils'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { Logger } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.commands.openChannel')

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

  protected async validateAmountToFund(amountToFund: BN): Promise<void> {
    const myAvailableTokens = await this.node.getBalance()

    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    }
  }

  public async open(state: GlobalState, counterpartyStr: string, amountToFundStr: string): Promise<string> {
    let counterparty: PeerId
    try {
      counterparty = await checkPeerIdInput(counterpartyStr, state)
    } catch (err) {
      log.error('Error while checking peerId', err)
      return styleValue(err.message, 'failure')
    }

    const amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
    await this.validateAmountToFund(amountToFund)

    try {
      const { channelId } = await this.node.openChannel(counterparty, amountToFund)
      return `${chalk.green(`Successfully opened channel`)} ${styleValue(channelId.toHex(), 'hash')}`
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }

  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  public async execute(query: string, state: GlobalState): Promise<string> {
    const [err, counterPartyB58Str, amountToFundStr] = this._assertUsage(query, [
      "counterParty's PeerId",
      'amountToFund'
    ])
    if (err) return styleValue(err, 'failure')

    return this.open(state, counterPartyB58Str, amountToFundStr)
  }
}
