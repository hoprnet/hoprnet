import Hopr from '@hoprnet/hopr-core'
import { Balance, moveDecimalPoint } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import chalk from 'chalk'
import PeerId from 'peer-id'
import { GlobalState } from '../../abstractCommand'
import { checkPeerIdInput, styleValue } from '../../utils'

export const openChannel = async ({
  counterpartyPeerId,
  amountToFundStr,
  state,
  node,
  log
}: {
  counterpartyPeerId: string
  amountToFundStr: string
  state: GlobalState
  node: Hopr
  log?: (string) => void
}) => {
  let counterparty: PeerId
  try {
    counterparty = checkPeerIdInput(counterpartyPeerId, state)
  } catch (err) {
    log && log(styleValue(err.message, 'failure'))
    return new Error('InvalidPeerId')
  }

  const amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
  const myAvailableTokens = await node.getBalance()
  if (amountToFund.lten(0)) {
    log && log(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    return new Error('InvalidAmountToFund')
  } else if (amountToFund.gt(myAvailableTokens.toBN())) {
    log && log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    return new Error('notEnoughFunds')
  }

  log && log('Opening channel...')

  try {
    const { channelId } = await node.openChannel(counterparty, amountToFund)
    log && log(`${chalk.green(`Successfully opened channel`)} ${styleValue(channelId.toHex(), 'hash')}`)
    return
  } catch (err) {
    log && log(styleValue(err.message, 'failure'))
    return new Error('failure')
  }
}
