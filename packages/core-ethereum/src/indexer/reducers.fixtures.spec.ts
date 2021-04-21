import type { Event } from './types'
import BN from 'bn.js'
import { BigNumber } from 'ethers'
import { Address, ChannelEntry, AccountEntry } from '../types'
import { partyA, partyB, secret1, secret2 } from './fixtures'

export const ACCOUNT_INITIALIZED_EVENT = {
  event: 'AccountInitialized',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: partyA.toAddress().toHex(),
    uncompressedPubKey: partyA.toUncompressedPubKeyHex(),
    secret: secret1.toHex()
  }
} as Event<'AccountInitialized'>

export const ACCOUNT_SECRET_UPDATED_EVENT = {
  event: 'AccountSecretUpdated',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: partyA.toAddress().toHex(),
    secret: secret2.toHex(),
    counter: BigNumber.from('2')
  }
} as Event<'AccountSecretUpdated'>

export const FUNDED_EVENT = {
  event: 'ChannelFunded',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    funder: 'funder',
    accountA: partyA.toAddress().toHex(),
    accountB: partyB.toAddress().toHex(),
    deposit: BigNumber.from('3'),
    partyABalance: BigNumber.from('3')
  }
} as Event<'ChannelFunded'>

export const FUNDED_EVENT_2 = {
  event: 'ChannelFunded',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    funder: 'funder',
    accountA: partyA.toAddress().toHex(),
    accountB: partyB.toAddress().toHex(),
    deposit: BigNumber.from('7'),
    partyABalance: BigNumber.from('0')
  }
} as Event<'ChannelFunded'>

export const OPENED_EVENT = {
  event: 'ChannelOpened',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    opener: partyA.toAddress().toHex(),
    counterparty: partyB.toHex()
  }
} as Event<'ChannelOpened'>

export const REDEEMED_EVENT = {
  event: 'TicketRedeemed',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    redeemer: partyA.toAddress().toHex(),
    counterparty: partyB.toAddress().toHex(),
    amount: BigNumber.from('1')
  }
} as Event<'TicketRedeemed'>

export const CLOSING_EVENT = {
  event: 'ChannelPendingToClose',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    initiator: partyA.toAddress().toHex(),
    counterparty: partyB.toAddress().toHex(),
    closureTime: BigNumber.from('1611671775')
  }
} as Event<'ChannelPendingToClose'>

export const REDEEMED_EVENT_2 = {
  event: 'TicketRedeemed',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    redeemer: partyB.toAddress().toHex(),
    counterparty: partyA.toAddress().toHex(),
    amount: BigNumber.from('2')
  }
} as Event<'TicketRedeemed'>

export const CLOSED_EVENT = {
  event: 'ChannelClosed',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    initiator: partyA.toAddress().toHex(),
    counterparty: partyB.toAddress().toHex(),
    partyAAmount: BigNumber.from('3'),
    partyBAmount: BigNumber.from('7')
  }
} as Event<'ChannelClosed'>

export const EMPTY_ACCOUNT = new AccountEntry(new Address(new Uint8Array()))

export const INITIALIZED_ACCOUNT = new AccountEntry(partyA.toAddress(), partyA, secret1, new BN(1))

export const SECRET_UPDATED_ACCOUNT = new AccountEntry(partyA.toAddress(), partyA, secret2, new BN(2))

export const EMPTY_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(0),
  partyABalance: new BN(0),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const FUNDED_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(3),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const FUNDED_CHANNEL_2 = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(10),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const OPENED_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(10),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const REDEEMED_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(10),
  partyABalance: new BN(4),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const CLOSING_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(10),
  partyABalance: new BN(4),
  closureTime: new BN(1611671775),
  stateCounter: new BN(2),
  closureByPartyA: true,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const REDEEMED_CHANNEL_2 = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(10),
  partyABalance: new BN(2),
  closureTime: new BN(1611671775),
  stateCounter: new BN(2),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const CLOSED_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(0),
  partyABalance: new BN(0),
  closureTime: new BN(0),
  stateCounter: new BN(10),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})
