import type { Event } from './types'
import BN from 'bn.js'
import assert from 'assert'
import { Multiaddr } from 'multiaddr'
import { BigNumber } from 'ethers'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { PublicKey, Hash, AccountEntry, ChannelEntry, u8aToHex } from '@hoprnet/hopr-utils'

export const partyA = PublicKey.fromString('0x03362b7b26bddb151a03056422d37119eab3a716562b6c3efdc62dec1540c9b091')
export const partyB = PublicKey.fromString('0x03217f3cd4d0b4b82997b25d1b6b68a933929fed724531cb30bbfd4729dc6b44e0')
export const secret1 = new Hash(stringToU8a('0xb8b37f62ec82443e5b5557c5a187fe3686790620cc04c06187c48f8636caac89'))
export const secret2 = new Hash(stringToU8a('0x294549f8629f0eeb2b8e01aca491f701f5386a9662403b485c4efe7d447dfba3'))
export const partyAMultiAddr = new Multiaddr(
  '/ip4/34.65.237.196/tcp/9091/p2p/16Uiu2HAmGJSpah8otZ92EouCVzqBb96g64iE5Xx3Rh6YDnTJL5Bv'
)

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
    account: partyA.toAddress().toHex(),
    multiaddr: u8aToHex(partyAMultiAddr.bytes)
  }
} as Event<'Announcement'>

export const PARTY_A_INITIALIZED_ACCOUNT = new AccountEntry(partyA.toAddress(), partyAMultiAddr, new BN(1))

export const FUNDED_EVENT = {
  event: 'ChannelUpdate',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    partyA: partyA.toAddress().toHex(),
    partyB: partyB.toAddress().toHex(),
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
    partyA: partyA.toAddress().toHex(),
    partyB: partyB.toAddress().toHex(),
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
