import type { Event } from './types'
import BN from 'bn.js'
import { BigNumber } from 'ethers'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Address, PublicKey, ChannelEntry, AccountEntry, Hash } from '../types'

const partyAUncompressedPubKey =
  '0x362b7b26bddb151a03056422d37119eab3a716562b6c3efdc62dec1540c9b0917c39b619ac36da7c9c02995f124df4353e69c226696857155d44a34744fd2327'
const partyAPubKey = PublicKey.fromString('0x03362b7b26bddb151a03056422d37119eab3a716562b6c3efdc62dec1540c9b091')
const partyA = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
// const partyBPubKey = Public.fromString('0x03217f3cd4d0b4b82997b25d1b6b68a933929fed724531cb30bbfd4729dc6b44e0')
const partyB = Address.fromString('0xbbCFC0fA0EBaa540e741dCA297368B2000089E2E')

const secret1 = new Hash(stringToU8a('0xb8b37f62ec82443e5b5557c5a187fe3686790620cc04c06187c48f8636caac89')) // secret1
const secret2 = new Hash(stringToU8a('0x294549f8629f0eeb2b8e01aca491f701f5386a9662403b485c4efe7d447dfba3')) // secret2

export const ACCOUNT_INITIALIZED_EVENT = {
  event: 'AccountInitialized',
  transactionHash: '',
  blockNumber: 0,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: partyA.toHex(),
    uncompressedPubKey: partyAUncompressedPubKey,
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
    account: partyA.toHex(),
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
    accountA: partyA.toHex(),
    accountB: partyB.toHex(),
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
    accountA: partyA.toHex(),
    accountB: partyB.toHex(),
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
    opener: partyA.toHex(),
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
    redeemer: partyA.toHex(),
    counterparty: partyB.toHex(),
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
    initiator: partyA.toHex(),
    counterparty: partyB.toHex(),
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
    redeemer: partyB.toHex(),
    counterparty: partyA.toHex(),
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
    initiator: partyA.toHex(),
    counterparty: partyB.toHex(),
    partyAAmount: BigNumber.from('3'),
    partyBAmount: BigNumber.from('7')
  }
} as Event<'ChannelClosed'>

export const EMPTY_ACCOUNT = new AccountEntry(new Address(new Uint8Array()))

export const INITIALIZED_ACCOUNT = new AccountEntry(partyA, partyAPubKey, secret1, new BN(1))

export const SECRET_UPDATED_ACCOUNT = new AccountEntry(partyA, partyAPubKey, secret2, new BN(2))

export const EMPTY_CHANNEL = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(0),
  partyABalance: new BN(0),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const FUNDED_CHANNEL = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(3),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const FUNDED_CHANNEL_2 = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(10),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const OPENED_CHANNEL = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(10),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const REDEEMED_CHANNEL = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(10),
  partyABalance: new BN(4),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const CLOSING_CHANNEL = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(10),
  partyABalance: new BN(4),
  closureTime: new BN(1611671775),
  stateCounter: new BN(2),
  closureByPartyA: true,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const REDEEMED_CHANNEL_2 = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(10),
  partyABalance: new BN(2),
  closureTime: new BN(1611671775),
  stateCounter: new BN(2),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})

export const CLOSED_CHANNEL = ChannelEntry.fromObject({
  partyA,
  partyB,
  deposit: new BN(0),
  partyABalance: new BN(0),
  closureTime: new BN(0),
  stateCounter: new BN(10),
  closureByPartyA: false,
  openedAt: new BN(0),
  closedAt: new BN(0)
})
