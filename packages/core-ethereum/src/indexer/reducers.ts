import type { Event } from './types'
import assert from 'assert'
import BN from 'bn.js'
import { publicKeyConvert } from 'secp256k1'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { AccountEntry, Address, Public, Hash, ChannelEntry } from '../types'
import { isPartyA } from '../utils'

export const onAccountInitialized = async (event: Event<'AccountInitialized'>): Promise<AccountEntry> => {
  const data = event.returnValues
  const address = Address.fromString(data.account)
  // library requires identifier TODO: investigate why
  const pubKey = new Public(publicKeyConvert(stringToU8a('0x04' + data.uncompressedPubKey.slice(2)), true))
  const secret = new Hash(stringToU8a(data.secret))
  const counter = new BN(1)

  return new AccountEntry(address, pubKey, secret, counter)
}

export const onAccountSecretUpdated = async (
  event: Event<'AccountSecretUpdated'>,
  storedAccount: AccountEntry
): Promise<AccountEntry> => {
  assert(storedAccount.isInitialized(), "'onAccountSecretUpdated' failed because account is not initialized")

  const data = event.returnValues
  const secret = new Hash(stringToU8a(data.secret))
  const counter = new BN(data.counter) // TODO: depend on indexer to increment this

  return new AccountEntry(storedAccount.address, storedAccount.publicKey, secret, counter)
}

export const onChannelFunded = async (
  event: Event<'ChannelFunded'>,
  channelEntry?: ChannelEntry
): Promise<ChannelEntry> => {
  const data = event.returnValues

  const accountA = Address.fromString(data.accountA)
  const accountB = Address.fromString(data.accountB)
  const parties: [Address, Address] = [accountA, accountB]

  if (channelEntry) {
    const deposit = channelEntry.deposit.add(new BN(data.deposit))
    const partyABalance = channelEntry.partyABalance.add(new BN(data.partyABalance))
    const closureTime = new BN(0)
    const stateCounter = channelEntry.stateCounter
    const closureByPartyA = false

    return new ChannelEntry(
      parties,
      deposit,
      partyABalance,
      closureTime,
      stateCounter,
      closureByPartyA,
      channelEntry.openedAt,
      channelEntry.closedAt
    )
  } else {
    const deposit = new BN(data.deposit)
    const partyABalance = new BN(data.partyABalance)
    const closureTime = new BN(0)
    const stateCounter = new BN(0)
    const closureByPartyA = false
    const openedAt = new BN(0)
    const closedAt = new BN(0)

    return new ChannelEntry(
      parties,
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
    channelEntry.parties,
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

  const data = event.returnValues
  const redeemer = Address.fromString(data.redeemer)
  const counterparty = Address.fromString(data.counterparty)
  const isRedeemerPartyA = isPartyA(redeemer, counterparty)
  const amount = new BN(data.amount)

  return new ChannelEntry(
    channelEntry.parties,
    channelEntry.deposit,
    isRedeemerPartyA ? channelEntry.partyABalance.add(amount) : channelEntry.partyABalance.sub(amount),
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

  const data = event.returnValues
  const initiator = Address.fromString(data.initiator)
  const counterparty = Address.fromString(data.counterparty)
  const isInitiatorPartyA = isPartyA(initiator, counterparty)

  return new ChannelEntry(
    channelEntry.parties,
    channelEntry.deposit,
    channelEntry.partyABalance,
    new BN(data.closureTime),
    channelEntry.stateCounter.addn(1),
    isInitiatorPartyA,
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

  return new ChannelEntry(
    channelEntry.parties,
    new BN(0),
    new BN(0),
    new BN(0),
    channelEntry.stateCounter.addn(8),
    false,
    channelEntry.openedAt,
    new BN(String(event.blockNumber))
  )
}
