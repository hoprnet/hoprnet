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
  assert.strictEqual(actual.deposit.toString(), expected.deposit.toString(), 'deposit')
  assert.strictEqual(actual.partyABalance.toString(), expected.partyABalance.toString(), 'partyABalance')
  assert.strictEqual(actual.closureTime.toString(), expected.closureTime.toString(), 'closureTime')
  assert.strictEqual(actual.stateCounter.toString(), expected.stateCounter.toString(), 'stateCounter')
  assert.strictEqual(actual.closureByPartyA, expected.closureByPartyA, 'closureByPartyA')
  assert.strictEqual(actual.openedAt.toString(), expected.openedAt.toString(), 'openedAt')
  assert.strictEqual(actual.closedAt.toString(), expected.closedAt.toString(), 'closedAt')
}

export const PARTY_A_INITIALIZED_EVENT = {
  event: 'AccountInitialized',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: partyA.toAddress().toHex(),
    uncompressedPubKey: partyA.toUncompressedPubKeyHex(),
    secret: secret1.toHex()
  }
} as Event<'AccountInitialized'>

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
