import type { Event } from './types'
import assert from 'assert'
import BN from 'bn.js'
import { BigNumber } from 'ethers'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { PublicKey, Hash, AccountEntry, ChannelEntry } from '../types'

export const partyA = PublicKey.fromString('0x03362b7b26bddb151a03056422d37119eab3a716562b6c3efdc62dec1540c9b091')
export const partyB = PublicKey.fromString('0x03217f3cd4d0b4b82997b25d1b6b68a933929fed724531cb30bbfd4729dc6b44e0')
export const secret1 = new Hash(stringToU8a('0xb8b37f62ec82443e5b5557c5a187fe3686790620cc04c06187c48f8636caac89'))
export const secret2 = new Hash(stringToU8a('0x294549f8629f0eeb2b8e01aca491f701f5386a9662403b485c4efe7d447dfba3'))

export const expectAccountsToBeEqual = (actual: AccountEntry, expected: AccountEntry) => {
  assert.strictEqual(actual.address.toString(), expected.address.toString(), 'address')
  assert.strictEqual(actual.publicKey.toString(), expected.publicKey.toString(), 'publicKey')
  assert.strictEqual(actual.secret.toString(), expected.secret.toString(), 'secret')
  assert.strictEqual(actual.counter.toString(), expected.counter.toString(), 'counter')
}

export const expectChannelsToBeEqual = (actual: ChannelEntry, expected: ChannelEntry) => {
  assert.strictEqual(actual.partyA.toHex(), expected.partyA.toHex(), 'partyA')
  assert.strictEqual(actual.partyB.toHex(), expected.partyB.toHex(), 'partyB')
  assert.strictEqual(actual.partyABalance.toBN().toString(), expected.partyABalance.toBN().toString(), 'partyABalance')
  assert.strictEqual(actual.partyBBalance.toBN().toString(), expected.partyBBalance.toBN().toString(), 'partyBBalance')
  assert.strictEqual(actual.commitmentPartyA.toHex(), expected.commitmentPartyA.toHex(), 'commitmentPartyA')
  assert.strictEqual(actual.commitmentPartyB.toHex(), expected.commitmentPartyB.toHex(), 'commitmentPartyB')
  assert.strictEqual(
    actual.partyATicketEpoch.toBN().toString(),
    expected.partyATicketEpoch.toBN().toString(),
    'partyATicketEpoch'
  )
  assert.strictEqual(
    actual.partyBTicketEpoch.toBN().toString(),
    expected.partyBTicketEpoch.toBN().toString(),
    'partyBTicketEpoch'
  )
  assert.strictEqual(
    actual.partyATicketIndex.toBN().toString(),
    expected.partyATicketIndex.toBN().toString(),
    'partyATicketIndex'
  )
  assert.strictEqual(
    actual.partyBTicketIndex.toBN().toString(),
    expected.partyBTicketIndex.toBN().toString(),
    'partyBTicketIndex'
  )
  assert.strictEqual(actual.status, expected.status, 'status')
  assert.strictEqual(actual.channelEpoch.toBN().toString(), expected.channelEpoch.toBN().toString(), 'channelEpoch')
  assert.strictEqual(actual.closureTime.toBN().toString(), expected.closureTime.toBN().toString(), 'closureTime')
  assert.strictEqual(actual.closureByPartyA, expected.closureByPartyA, 'closureByPartyA')
}

export const PARTY_A_INITIALIZED_EVENT = {
  event: 'ChannelUpdate',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: partyA.toAddress().toHex(),
    uncompressedPubKey: partyA.toUncompressedPubKeyHex(),
    secret: secret1.toHex()
  }
} as Event<'ChannelUpdate'>

export const PARTY_A_INITIALIZED_ACCOUNT = new AccountEntry(partyA.toAddress(), partyA, secret1, new BN(1))

export const FUNDED_EVENT = {
  event: 'ChannelFunded',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    funder: partyA.toAddress().toHex(),
    accountA: partyA.toAddress().toHex(),
    accountB: partyB.toAddress().toHex(),
    deposit: BigNumber.from('3'),
    partyABalance: BigNumber.from('3')
  }
} as Event<'ChannelFunded'>

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

export const OPENED_EVENT = {
  event: 'ChannelOpened',
  transactionHash: '',
  blockNumber: 3,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    opener: partyA.toAddress().toHex(),
    counterparty: partyB.toAddress().toHex()
  }
} as Event<'ChannelOpened'>

export const OPENED_CHANNEL = ChannelEntry.fromObject({
  partyA: partyA.toAddress(),
  partyB: partyB.toAddress(),
  deposit: new BN(3),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false,
  openedAt: new BN(3),
  closedAt: new BN(0)
})
