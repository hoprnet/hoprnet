import type { Event } from './types'
import assert from 'assert'
import BN from 'bn.js'
import { stringToU8a, u8aConcat } from '@hoprnet/hopr-utils'
import { AccountEntry, Address, PublicKey, Hash, ChannelEntry } from '../types'

export const onAccountInitialized = async (event: Event<'AccountInitialized'>): Promise<AccountEntry> => {
  const data = event.args
  const address = Address.fromString(data.account)
  // library requires identifier
  const pubKey = PublicKey.fromUncompressedPubKey(u8aConcat(new Uint8Array([4]), stringToU8a(data.uncompressedPubKey)))
  const secret = new Hash(stringToU8a(data.secret))
  const counter = new BN(1)

  return new AccountEntry(address, pubKey, secret, counter)
}

export const onAccountSecretUpdated = async (
  event: Event<'AccountSecretUpdated'>,
  storedAccount: AccountEntry
): Promise<AccountEntry> => {
  assert(storedAccount.isInitialized(), "'onAccountSecretUpdated' failed because account is not initialized")

  const data = event.args
  const secret = new Hash(stringToU8a(data.secret))
  const counter = new BN(data.counter.toString()) // TODO: depend on indexer to increment this

  return new AccountEntry(storedAccount.address, storedAccount.publicKey, secret, counter)
}

export const onChannelFunded = async (
  event: Event<'ChannelFunded'>,
  channelEntry?: ChannelEntry
): Promise<ChannelEntry> => {
  const data = event.args

  const accountA = Address.fromString(data.accountA)
  const accountB = Address.fromString(data.accountB)
  const [partyA, partyB] = accountA.sortPair(accountB)

  if (channelEntry) {
    const deposit = channelEntry.deposit.add(new BN(data.deposit.toString()))
    const partyABalance = channelEntry.partyABalance.add(new BN(data.partyABalance.toString()))
    const closureTime = new BN(0)
    const stateCounter = channelEntry.stateCounter
    const closureByPartyA = false

    return new ChannelEntry(
      partyA,
      partyB,
      deposit,
      partyABalance,
      closureTime,
      stateCounter,
      closureByPartyA,
      channelEntry.openedAt,
      channelEntry.closedAt
    )
  } else {
    const deposit = new BN(data.deposit.toString())
    const partyABalance = new BN(data.partyABalance.toString())
    const closureTime = new BN(0)
    const stateCounter = new BN(0)
    const closureByPartyA = false
    const openedAt = new BN(0)
    const closedAt = new BN(0)

    return new ChannelEntry(
      partyA,
      partyB,
      deposit,
      partyABalance,
      closureTime,
      stateCounter,
      closureByPartyA,
      openedAt,
      closedAt
    )
  }
}

export const onChannelOpened = async (
  event: Event<'ChannelOpened'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(channelEntry.getStatus() === 'CLOSED', "'onChannelOpened' failed because channel is not in 'CLOSED' status")

  return new ChannelEntry(
    channelEntry.partyA,
    channelEntry.partyB,
    channelEntry.deposit,
    channelEntry.partyABalance,
    channelEntry.closureTime,
    channelEntry.stateCounter.addn(1),
    false,
    new BN(String(event.blockNumber)),
    channelEntry.closedAt
  )
}

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

export const onChannelPendingToClose = async (
  event: Event<'ChannelPendingToClose'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(
    channelEntry.getStatus() === 'OPEN',
    "'onInitiatedChannelClosure' failed because channel is not in 'OPEN' status"
  )

  const data = event.args
  const initiator = Address.fromString(data.initiator)
  const counterparty = Address.fromString(data.counterparty)

  return new ChannelEntry(
    channelEntry.partyA,
    channelEntry.partyB,
    channelEntry.deposit,
    channelEntry.partyABalance,
    new BN(data.closureTime.toString()),
    channelEntry.stateCounter.addn(1),
    initiator.lt(counterparty),
    channelEntry.openedAt,
    channelEntry.closedAt
  )
}

export const onChannelClosed = async (
  event: Event<'ChannelClosed'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(
    channelEntry.getStatus() === 'PENDING_TO_CLOSE',
    "'onClosedChannel' failed because channel is not in 'PENDING_TO_CLOSE' status"
  )

  return ChannelEntry.fromObject({
    partyA: channelEntry.partyA,
    partyB: channelEntry.partyB,
    deposit: new BN(0),
    partyABalance: new BN(0),
    closureTime: new BN(0),
    stateCounter: channelEntry.stateCounter.addn(8),
    closureByPartyA: false,
    openedAt: channelEntry.openedAt,
    closedAt: new BN(String(event.blockNumber))
  })
}
