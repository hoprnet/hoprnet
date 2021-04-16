import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import chalk from 'chalk'
import BN from 'bn.js'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput, styleValue } from './utils'
import { Logger } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.commands.fundChannel')

export default class FundChannel extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'fund'
  }

  public help() {
    return 'Fund a channel, if channel is closed it will open it'
  }

  async execute(query: string, state: GlobalState): Promise<string | void> {
    if (query == null) {
      return styleValue(
        `Invalid arguments. Expected 'fund <peerId> <myFund> <counterpartyFund>'. Received '${query}'`,
        'failure'
      )
    }

    const [error, peerIdInput, myFundInput, counterpartyFundInput] = this._assertUsage(query, [
      'peerId',
      'myFund',
      'counterpartyFund'
    ])
    if (error) return styleValue(error, 'failure')

    let peerId: PeerId
    let myFund: BN
    let counterpartyFund: BN

    try {
      peerId = await checkPeerIdInput(peerIdInput, state)
      if (isNaN(Number(myFundInput))) throw Error('Argument <myFund> is not a number')
      myFund = new BN(moveDecimalPoint(myFundInput, Balance.DECIMALS))
      if (isNaN(Number(counterpartyFundInput))) throw Error('Argument <counterpartyFund> is not a number')
      counterpartyFund = new BN(moveDecimalPoint(counterpartyFundInput, Balance.DECIMALS))
    } catch (err) {
      log.error('Error while checking peerId', err)
      return styleValue(err.message, 'failure')
    }

    try {
      const { channelId } = await this.node.fundChannel(peerId, myFund, counterpartyFund)
      return `${chalk.green(`Successfully funded channel`)} ${styleValue(channelId.toHex(), 'hash')}`
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
