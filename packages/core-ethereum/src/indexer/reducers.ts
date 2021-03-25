import type { Event } from './types'
import assert from 'assert'
import BN from 'bn.js'
import Web3 from 'web3'
import { Account, AccountId, Public, Hash, ChannelEntry } from '../types'
import { isPartyA } from '../utils'

const { hexToBytes, hexToNumberString } = Web3.utils

export const onAccountInitialized = async (event: Event<'AccountInitialized'>): Promise<Account> => {
  const data = event.returnValues
  const pubKey = new Public([...hexToBytes(data.pubKeyFirstHalf), ...hexToBytes(data.pubKeySecondHalf)])
  const secret = new Hash(hexToBytes(data.secret))
  const counter = new BN(0)

  return new Account(pubKey, secret, counter)
}

export const onAccountSecretUpdated = async (
  event: Event<'AccountSecretUpdated'>,
  storedAccount: Account
): Promise<Account> => {
  const data = event.returnValues
  const secret = new Hash(hexToBytes(data.secret))
  const counter = new BN(hexToNumberString(data.secret)) // TODO: depend on indexer to increment this

  return new Account(storedAccount.publicKey, secret, counter)
}

export const onChannelFunded = async (
  event: Event<'ChannelFunded'>,
  channelEntry?: ChannelEntry
): Promise<ChannelEntry> => {
  const data = event.returnValues

  const accountA = new AccountId(hexToBytes(data.accountA))
  const accountB = new AccountId(hexToBytes(data.accountB))
  const parties: [AccountId, AccountId] = [accountA, accountB]

  if (channelEntry) {
    const status = channelEntry.getStatus()
    assert(
      status === 'UNINITIALISED' || status === 'FUNDED',
      "'onFundedChannel' failed because channel is not in 'UNINITIALISED' or 'FUNDED' status"
    )

    const deposit = channelEntry.deposit.add(new BN(data.deposit))
    const partyABalance = channelEntry.partyABalance.add(new BN(data.partyABalance))
    const closureTime = new BN(0)
    const stateCounter = status === 'FUNDED' ? channelEntry.stateCounter : channelEntry.stateCounter.addn(1)
    const closureByPartyA = false

    return new ChannelEntry(parties, deposit, partyABalance, closureTime, stateCounter, closureByPartyA)
  } else {
    const deposit = new BN(data.deposit)
    const partyABalance = new BN(data.partyABalance)
    const closureTime = new BN(0)
    const stateCounter = new BN(1)
    const closureByPartyA = false

    return new ChannelEntry(parties, deposit, partyABalance, closureTime, stateCounter, closureByPartyA)
  }
}

export const onChannelOpened = async (
  _event: Event<'ChannelOpened'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(channelEntry.getStatus() === 'FUNDED', "'onOpenedChannel' failed because channel is not in 'FUNDED' status")

  return new ChannelEntry(
    channelEntry.parties,
    channelEntry.deposit,
    channelEntry.partyABalance,
    channelEntry.closureTime,
    channelEntry.stateCounter.addn(1),
    false
  )
}

export const onTicketRedeemed = async (
  event: Event<'TicketRedeemed'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  const status = channelEntry.getStatus()

  assert(
    status === 'OPEN' || status === 'PENDING',
    "'onRedeemedTicket' failed because channel is not in 'OPEN' or 'PENDING' status"
  )

  const data = event.returnValues
  const redeemerAccountId = new AccountId(hexToBytes(data.redeemer))
  const counterpartyAccountId = new AccountId(hexToBytes(data.counterparty))
  const isRedeemerPartyA = isPartyA(redeemerAccountId, counterpartyAccountId)

  return new ChannelEntry(
    channelEntry.parties,
    channelEntry.deposit,
    isRedeemerPartyA
      ? channelEntry.partyABalance.add(new BN(data.amount))
      : channelEntry.partyABalance.sub(new BN(data.amount)),
    channelEntry.closureTime,
    channelEntry.stateCounter,
    false
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
  const initiatorAccountId = new AccountId(hexToBytes(data.initiator))
  const counterpartyAccountId = new AccountId(hexToBytes(data.counterparty))
  const isInitiatorPartyA = isPartyA(initiatorAccountId, counterpartyAccountId)

  return new ChannelEntry(
    channelEntry.parties,
    channelEntry.deposit,
    channelEntry.partyABalance,
    new BN(hexToNumberString(data.closureTime)),
    channelEntry.stateCounter.addn(1),
    isInitiatorPartyA
  )
}

export const onChannelClosed = async (
  _event: Event<'ChannelClosed'>,
  channelEntry: ChannelEntry
): Promise<ChannelEntry> => {
  assert(channelEntry.getStatus() === 'PENDING', "'onClosedChannel' failed because channel is not in 'PENDING' status")

  return new ChannelEntry(
    channelEntry.parties,
    new BN(0),
    new BN(0),
    new BN(0),
    channelEntry.stateCounter.addn(7),
    false
  )
}
