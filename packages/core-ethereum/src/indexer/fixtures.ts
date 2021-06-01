import type { Event } from './types'
import BN from 'bn.js'
import assert from 'assert'
import { BigNumber } from 'ethers'
import { Hash, AccountEntry, ChannelEntry, u8aToHex } from '@hoprnet/hopr-utils'
import { PARTY_A, PARTY_B, PARTY_A_MULTIADDR } from '../fixtures'

export * from '../fixtures'

export const expectAccountsToBeEqual = (actual: AccountEntry, expected: AccountEntry) => {
  assert.strictEqual(actual.address.toString(), expected.address.toString(), 'address')
  assert.strictEqual(actual.getPublicKey().toString(), expected.getPublicKey().toString(), 'publicKey')
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
  event: 'Announcement',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A.toAddress().toHex(),
    multiaddr: u8aToHex(PARTY_A_MULTIADDR.bytes)
  }
} as Event<'Announcement'>

export const PARTY_A_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_A.toAddress(), PARTY_A_MULTIADDR, new BN(1))

export const FUNDED_EVENT = {
  event: 'ChannelUpdate',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    partyA: PARTY_A.toAddress().toHex(),
    partyB: PARTY_B.toAddress().toHex(),
    newState: {
      partyABalance: BigNumber.from('3'),
      partyBBalance: BigNumber.from('0'),
      partyACommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      partyBCommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      partyATicketEpoch: BigNumber.from('0'),
      partyBTicketEpoch: BigNumber.from('0'),
      partyATicketIndex: BigNumber.from('0'),
      partyBTicketIndex: BigNumber.from('0'),
      status: 0,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0'),
      closureByPartyA: false
    }
  } as any
} as Event<'ChannelUpdate'>

export const FUNDED_CHANNEL = ChannelEntry.fromSCEvent(FUNDED_EVENT)

export const OPENED_EVENT = {
  event: 'ChannelUpdate',
  transactionHash: '',
  blockNumber: 3,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    partyA: PARTY_A.toAddress().toHex(),
    partyB: PARTY_B.toAddress().toHex(),
    newState: {
      partyABalance: BigNumber.from('3'),
      partyBBalance: BigNumber.from('0'),
      partyACommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      partyBCommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      partyATicketEpoch: BigNumber.from('0'),
      partyBTicketEpoch: BigNumber.from('0'),
      partyATicketIndex: BigNumber.from('0'),
      partyBTicketIndex: BigNumber.from('0'),
      status: 1,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0'),
      closureByPartyA: false
    }
  } as any
} as Event<'ChannelUpdate'>

export const OPENED_CHANNEL = ChannelEntry.fromSCEvent(OPENED_EVENT)
