import Hopr from '@hoprnet/hopr-core'
import { Balance, moveDecimalPoint } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { GlobalState } from '../../../commands/abstractCommand'
import { checkPeerIdInput } from '../../../commands/utils'

export const openChannel = async ({
  counterpartyPeerId,
  amountToFundStr,
  state,
  node
}: {
  counterpartyPeerId: string
  amountToFundStr: string
  state: GlobalState
  node: Hopr
}) => {
  let counterparty: PeerId
  try {
    counterparty = checkPeerIdInput(counterpartyPeerId, state)
  } catch (err) {
    return new Error('invalidPeerId')
  }

  let amountToFund: BN
  let myAvailableTokens: Balance
  try {
    amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
    myAvailableTokens = await node.getBalance()
  } catch (error) {
    return new Error('invalidAmountToFund')
  }

  if (amountToFund.lten(0)) {
    return new Error('invalidAmountToFund')
  } else if (amountToFund.gt(myAvailableTokens.toBN())) {
    return new Error(
      JSON.stringify({
        status: 'notEnoughFunds',
        tokensRequired: amountToFund.toString(10),
        currentBalance: myAvailableTokens.toBN().toString(10)
      })
    )
  }

  try {
    const { channelId } = await node.openChannel(counterparty, amountToFund)
    return channelId.toHex()
  } catch (err) {
    return new Error(err.message.includes('Channel is already opened') ? 'channelAlreadyOpen' : 'failure')
  }
}
