import type { Event } from './topics'
import assert from 'assert'
import BN from 'bn.js'
import { ChannelEntry } from '../types'
import { isPartyA } from '../utils'

export const onFundedChannel = async (
  event: Event<'FundedChannel'>,
  channelEntry?: ChannelEntry
): Promise<ChannelEntry> => {
  const recipientAccountId = await event.data.recipient.toAccountId()
  const counterpartyAccountId = await event.data.counterparty.toAccountId()
  const isRecipientPartyA = isPartyA(recipientAccountId, counterpartyAccountId)

  if (channelEntry) {
    assert(
      channelEntry.status === 'UNINITIALISED' || channelEntry.status === 'FUNDED',
      "'onFundedChannel' failed because channel is not in 'UNINITIALISED' or 'FUNDED' status"
    )

    return new ChannelEntry(undefined, {
      blockNumber: event.blockNumber,
      transactionIndex: event.transactionIndex,
      logIndex: event.logIndex,
      deposit: channelEntry.deposit.add(event.data.recipientAmount.add(event.data.counterpartyAmount)),
      partyABalance: channelEntry.partyABalance.add(
        isRecipientPartyA ? event.data.recipientAmount : event.data.counterpartyAmount
      ),
      closureTime: new BN(0),
      stateCounter: channelEntry.status === 'FUNDED' ? channelEntry.stateCounter : channelEntry.stateCounter.addn(1),
      closureByPartyA: false
    })
  } else {
    return new ChannelEntry(undefined, {
      blockNumber: event.blockNumber,
      transactionIndex: event.transactionIndex,
      logIndex: event.logIndex,
      deposit: event.data.recipientAmount.add(event.data.counterpartyAmount),
      partyABalance: isRecipientPartyA ? event.data.recipientAmount : event.data.counterpartyAmount,
      closureTime: new BN(0),
      stateCounter: new BN(1),
      closureByPartyA: false
    })
  }
}

export const onOpenedChannel = async (
  event: Event<'OpenedChannel'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(channelEntry.status === 'FUNDED', "'onOpenedChannel' failed because channel is not in 'FUNDED' status")

  return new ChannelEntry(undefined, {
    blockNumber: event.blockNumber,
    transactionIndex: event.transactionIndex,
    logIndex: event.logIndex,
    deposit: channelEntry.deposit,
    partyABalance: channelEntry.partyABalance,
    closureTime: channelEntry.closureTime,
    stateCounter: channelEntry.stateCounter.addn(1),
    closureByPartyA: false
  })
}

export const onRedeemedTicket = async (
  event: Event<'RedeemedTicket'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(
    channelEntry.status === 'OPEN' || channelEntry.status === 'PENDING',
    "'onRedeemedTicket' failed because channel is not in 'OPEN' or 'PENDING' status"
  )

  const redeemerAccountId = await event.data.redeemer.toAccountId()
  const counterpartyAccountId = await event.data.counterparty.toAccountId()
  const isRedeemerPartyA = isPartyA(redeemerAccountId, counterpartyAccountId)

  return new ChannelEntry(undefined, {
    blockNumber: event.blockNumber,
    transactionIndex: event.transactionIndex,
    logIndex: event.logIndex,
    deposit: channelEntry.deposit,
    partyABalance: isRedeemerPartyA
      ? channelEntry.partyABalance.add(event.data.amount)
      : channelEntry.partyABalance.sub(event.data.amount),
    closureTime: channelEntry.closureTime,
    stateCounter: channelEntry.stateCounter,
    closureByPartyA: false
  })
}

export const onInitiatedChannelClosure = async (
  event: Event<'InitiatedChannelClosure'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  try {
    assert(channelEntry.status === 'OPEN', "'onInitiatedChannelClosure' failed because channel is not in 'OPEN' status")
  } catch (err) {
    console.log({
      transactionHash: event.transactionHash,
      blockNumber: event.blockNumber.toString(),
      transactionIndex: event.transactionIndex.toString(),
      logIndex: event.logIndex.toString(),
      initiator: event.data.initiator.toHex(),
      counterparty: event.data.counterparty.toHex()
    })
    console.log({
      blockNumber: channelEntry.blockNumber.toString(),
      transactionIndex: channelEntry.transactionIndex.toString(),
      logIndex: channelEntry.logIndex.toString(),
      deposit: channelEntry.deposit.toString(),
      partyABalance: channelEntry.partyABalance.toString(),
      closureTime: channelEntry.closureTime.toString(),
      stateCounter: channelEntry.stateCounter.toString(),
      closureByPartyA: channelEntry.closureByPartyA
    })
    console.error(err)
    throw err
  }

  const initiatorAccountId = await event.data.initiator.toAccountId()
  const counterpartyAccountId = await event.data.counterparty.toAccountId()
  const isInitiatorPartyA = isPartyA(initiatorAccountId, counterpartyAccountId)

  return new ChannelEntry(undefined, {
    blockNumber: event.blockNumber,
    transactionIndex: event.transactionIndex,
    logIndex: event.logIndex,
    deposit: channelEntry.deposit,
    partyABalance: channelEntry.partyABalance,
    closureTime: event.data.closureTime,
    stateCounter: channelEntry.stateCounter.addn(1),
    closureByPartyA: isInitiatorPartyA
  })
}

export const onClosedChannel = async (
  event: Event<'ClosedChannel'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(channelEntry.status === 'PENDING', "'onClosedChannel' failed because channel is not in 'PENDING' status")

  return new ChannelEntry(undefined, {
    blockNumber: event.blockNumber,
    transactionIndex: event.transactionIndex,
    logIndex: event.logIndex,
    deposit: new BN(0),
    partyABalance: new BN(0),
    closureTime: new BN(0),
    stateCounter: channelEntry.stateCounter.addn(7),
    closureByPartyA: false
  })
}
