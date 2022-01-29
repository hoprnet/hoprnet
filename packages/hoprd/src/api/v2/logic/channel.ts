import Hopr from '@hoprnet/hopr-core'
import { Balance, ChannelStatus, moveDecimalPoint, PublicKey } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import PeerId from 'peer-id'
import { checkPeerIdInput } from '../../../commands/utils'
import { APIv2State } from '../../v2'

export const openChannel = async ({
  counterpartyPeerId,
  amountToFundStr,
  state,
  node
}: {
  counterpartyPeerId: string
  amountToFundStr: string
  state: APIv2State
  node: Hopr
}) => {
  let counterparty: PeerId
  try {
    counterparty = checkPeerIdInput(counterpartyPeerId, state as any)
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

export interface ChannelInfo {
  type: 'outgoing' | 'incoming'
  channelId: string
  peerId: string
  status: string
  balance: string
}

const channelStatusToString = (status: ChannelStatus): string => {
  if (status === 0) return 'Closed'
  else if (status === 1) return 'WaitingForCommitment'
  else if (status === 2) return 'Open'
  else if (status === 3) return 'PendingToClose'
  return 'Unknown'
}

export const listOpenChannels = async ({ node }: { node: Hopr }) => {
  try {
    const selfPubKey = new PublicKey(node.getId().pubKey.marshal())
    const selfAddress = selfPubKey.toAddress()

    const channelsFrom: ChannelInfo[] = (await node.getChannelsFrom(selfAddress))
      .filter((channel) => channel.status !== ChannelStatus.Closed)
      .map(({ /*getId,*/ destination, status, balance }) => ({
        type: 'incoming',
        channelId: 'getId().toHex()', // <<<--- NOTE: issue here, source is undefined
        peerId: destination.toPeerId().toB58String(),
        status: channelStatusToString(status),
        balance: balance.toFormattedString()
      }))

    const channelsTo: ChannelInfo[] = (await node.getChannelsTo(selfAddress))
      .filter((channel) => channel.status !== ChannelStatus.Closed)
      .map(({ /*getId,*/ source, status, balance }) => ({
        type: 'outgoing',
        channelId: 'getId().toHex()', // <<<--- NOTE: issue here, source is undefined
        peerId: source.toPeerId().toB58String(),
        status: channelStatusToString(status),
        balance: balance.toFormattedString()
      }))

    return { incoming: channelsFrom, outgoing: channelsTo }
  } catch (err) {
    return new Error(err.message)
  }
}
