import type { Event } from './types'
import BN from 'bn.js'
import assert from 'assert'
import { BigNumber } from 'ethers'
import { Hash, AccountEntry, ChannelEntry, u8aToHex } from '@hoprnet/hopr-utils'
import { PARTY_A, PARTY_B, PARTY_A_MULTIADDR, PARTY_B_MULTIADDR } from '../fixtures'

export * from '../fixtures'

export const expectAccountsToBeEqual = (actual: AccountEntry, expected: AccountEntry) => {
  assert(actual, 'account is null')
  assert.strictEqual(actual.address.toString(), expected.address.toString(), 'address')
  assert.strictEqual(actual.getPublicKey().toString(), expected.getPublicKey().toString(), 'publicKey')
}

export const expectChannelsToBeEqual = (actual: ChannelEntry, expected: ChannelEntry) => {
  assert(actual, 'channel is null')
  assert.strictEqual(actual.source.toHex(), expected.source.toHex(), 'source')
  assert.strictEqual(actual.destination.toHex(), expected.destination.toHex(), 'destination')
  assert.strictEqual(actual.balance.toBN().toString(), expected.balance.toBN().toString(), 'balance')
  assert.strictEqual(actual.commitment.toHex(), expected.commitment.toHex(), 'commitment')
  assert.strictEqual(actual.ticketEpoch.toBN().toString(), expected.ticketEpoch.toBN().toString(), 'ticketEpoch')
  assert.strictEqual(actual.ticketIndex.toBN().toString(), expected.ticketIndex.toBN().toString(), 'ticketIndex')
  assert.strictEqual(actual.status, expected.status, 'status')
  assert.strictEqual(actual.channelEpoch.toBN().toString(), expected.channelEpoch.toBN().toString(), 'channelEpoch')
  assert.strictEqual(actual.closureTime.toBN().toString(), expected.closureTime.toBN().toString(), 'closureTime')
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

export const PARTY_B_INITIALIZED_EVENT = {
  event: 'Announcement',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_B.toAddress().toHex(),
    multiaddr: u8aToHex(PARTY_B_MULTIADDR.bytes)
  }
} as Event<'Announcement'>

export const PARTY_A_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_A.toAddress(), PARTY_A_MULTIADDR, new BN(1))

export const PARTY_B_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_B.toAddress(), PARTY_B_MULTIADDR, new BN(1))

export const OPENED_EVENT = {
  event: 'ChannelUpdate',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    source: PARTY_A.toAddress().toHex(),
    destination: PARTY_B.toAddress().toHex(),
    newState: {
      balance: BigNumber.from('3'),
      commitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      ticketEpoch: BigNumber.from('0'),
      ticketIndex: BigNumber.from('0'),
      status: 1,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0')
    }
  } as any
} as Event<'ChannelUpdate'>
