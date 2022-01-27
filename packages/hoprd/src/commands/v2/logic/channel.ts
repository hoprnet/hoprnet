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

  let amountToFund: BN
  let myAvailableTokens: Balance
  try {
    amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
    myAvailableTokens = await node.getBalance()
  } catch (error) {
    log && log('Invalid amount to fund')
    return new Error('invalidAmountToFund')
  }

  if (amountToFund.lten(0)) {
    log && log(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    return new Error('invalidAmountToFund')
  } else if (amountToFund.gt(myAvailableTokens.toBN())) {
    log && log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    return new Error(
      JSON.stringify({
        status: 'notEnoughFunds',
        tokensRequired: amountToFund.toString(10),
        currentBalance: myAvailableTokens.toBN().toString(10)
      })
    )
  }

  log && log('Opening channel...')

  try {
    const { channelId } = await node.openChannel(counterparty, amountToFund)
    log && log(`${chalk.green(`Successfully opened channel`)} ${styleValue(channelId.toHex(), 'hash')}`)
    return channelId.toHex()
  } catch (err) {
    log && log(styleValue(err.message, 'failure'))
    return new Error(err.message.includes('Channel is already opened') ? 'channelAlreadyOpen' : 'failure')
  }
}
