import type Hopr from '@hoprnet/hopr-core'
import type { PeerId } from '@libp2p/interface-peer-id'
import chalk from 'chalk'
import BN from 'bn.js'
import { moveDecimalPoint, Balance } from '@hoprnet/hopr-utils'
import type { StateOps } from '../types.js'
import { AbstractCommand } from './abstractCommand.js'
import { checkPeerIdInput, styleValue } from './utils/index.js'

export default class FundChannel extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'fund'
  }

  public help() {
    return '(deprecated) Fund a channel, if channel is closed it will open it'
  }

  async execute(log, query: string, { getState }: StateOps): Promise<void> {
    if (query == null) {
      return log(
        styleValue(
          `Invalid arguments. Expected 'fund <peerId> <myFund> <counterpartyFund>'. Received '${query}'`,
          'failure'
        )
      )
    }

    const [error, peerIdInput, myFundInput, counterpartyFundInput] = this._assertUsage(query, [
      'peerId',
      'myFund',
      'counterpartyFund'
    ])
    if (error) return log(styleValue(error, 'failure'))

    let peerId: PeerId
    let myFund: BN
    let counterpartyFund: BN

    try {
      peerId = checkPeerIdInput(peerIdInput, getState())
      if (isNaN(Number(myFundInput))) throw Error('Argument <myFund> is not a number')
      myFund = new BN(moveDecimalPoint(myFundInput, Balance.DECIMALS))
      if (isNaN(Number(counterpartyFundInput))) throw Error('Argument <counterpartyFund> is not a number')
      counterpartyFund = new BN(moveDecimalPoint(counterpartyFundInput, Balance.DECIMALS))
    } catch (err) {
      return log(styleValue(err instanceof Error ? err.message : 'Unknown error', 'failure'))
    }

    log('Funding channel...')

    try {
      await this.node.fundChannel(peerId, myFund, counterpartyFund)
      return log(`${chalk.green(`Successfully funded channel`)} to ${peerId.toString()}`)
    } catch (err) {
      return log(styleValue(err instanceof Error ? err.message : 'Unknown error', 'failure'))
    }
  }
}
