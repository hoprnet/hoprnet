import type { Event } from './types'
import assert from 'assert'
import BN from 'bn.js'
import {Address, ChannelEntry } from '../types'


export const onTicketRedeemed = async (
  event: Event<'TicketRedeemed'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  const status = channelEntry.getStatus()

  assert(
    status === 'OPEN' || status === 'PENDING_TO_CLOSE',
    "'onRedeemedTicket' failed because channel is not in 'OPEN' or 'PENDING' status"
  )

  const data = event.args
  const redeemer = Address.fromString(data.redeemer)
  const counterparty = Address.fromString(data.counterparty)
  const amount = new BN(data.amount.toString())

  return new ChannelEntry(
    channelEntry.partyA,
    channelEntry.partyB,
    channelEntry.deposit,
    redeemer.lt(counterparty) ? channelEntry.partyABalance.add(amount) : channelEntry.partyABalance.sub(amount),
    channelEntry.closureTime,
    channelEntry.stateCounter,
    false,
    channelEntry.openedAt,
    channelEntry.closedAt
  )
}
