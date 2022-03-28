import type { Event, TokenEvent, RegistryEvent } from './types'
import BN from 'bn.js'
import assert from 'assert'
import { BigNumber } from 'ethers'
import {
  Hash,
  AccountEntry,
  ChannelEntry,
  u8aToHex,
  Ticket,
  Challenge,
  stringToU8a,
  UINT256,
  Balance,
  Signature,
  SIGNATURE_LENGTH
} from '@hoprnet/hopr-utils'
import { PARTY_A, PARTY_B, PARTY_A_MULTIADDR, PARTY_B_MULTIADDR } from '../fixtures'

export * from '../fixtures'

export const expectAccountsToBeEqual = (actual: AccountEntry, expected: AccountEntry) => {
  assert(actual, 'account is null')
  assert(actual.publicKey.eq(expected.publicKey), 'publicKey')
}

export const expectChannelsToBeEqual = (actual: ChannelEntry, expected: ChannelEntry) => {
  assert(actual, 'channel is null')
  assert(actual.source.eq(expected.source), 'source')
  assert(actual.destination.eq(expected.destination), 'destination')
  assert(actual.balance.eq(expected.balance), 'balance')
  assert(actual.commitment.eq(expected.commitment), 'commitment')
  assert(actual.ticketEpoch.eq(expected.ticketEpoch), 'ticketEpoch')
  assert(actual.ticketIndex.eq(expected.ticketIndex), 'ticketIndex')
  assert(actual.status == expected.status, 'status')
  assert(actual.channelEpoch.eq(expected.channelEpoch), 'channelEpoch')
  assert(actual.closureTime.eq(expected.closureTime), 'closureTime')
}

export const PARTY_A_INITIALIZED_EVENT = {
  event: 'Announcement',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A.toAddress().toHex(),
    publicKey: PARTY_A.toUncompressedPubKeyHex(),
    multiaddr: u8aToHex(PARTY_A_MULTIADDR.bytes)
  }
} as Event<'Announcement'>

export const PARTY_B_INITIALIZED_EVENT = {
  event: 'Announcement',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 1,
  logIndex: 0,
  args: {
    account: PARTY_B.toAddress().toHex(),
    publicKey: PARTY_B.toUncompressedPubKeyHex(),
    multiaddr: u8aToHex(PARTY_B_MULTIADDR.bytes)
  }
} as Event<'Announcement'>

export const PARTY_A_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_A, PARTY_A_MULTIADDR, new BN(1))

export const PARTY_B_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_B, PARTY_B_MULTIADDR, new BN(1))

export const OPENED_EVENT = {
  event: 'ChannelUpdated',
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
} as Event<'ChannelUpdated'>

export const COMMITTED_EVENT = {
  event: 'ChannelUpdated',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 10,
  args: {
    source: PARTY_A.toAddress().toHex(),
    destination: PARTY_B.toAddress().toHex(),
    newState: {
      balance: BigNumber.from('3'),
      commitment: new Hash(new Uint8Array({ length: Hash.SIZE }).fill(1)).toHex(),
      ticketEpoch: BigNumber.from('0'),
      ticketIndex: BigNumber.from('0'),
      status: 2,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0')
    }
  } as any
} as Event<'ChannelUpdated'>

export const UPDATED_WHEN_REDEEMED_EVENT = {
  event: 'ChannelUpdated',
  transactionHash: '',
  blockNumber: 5,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    source: PARTY_A.toAddress().toHex(),
    destination: PARTY_B.toAddress().toHex(),
    newState: {
      balance: BigNumber.from('1'),
      commitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      ticketEpoch: BigNumber.from('0'),
      ticketIndex: BigNumber.from('1'),
      status: 2,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0')
    }
  } as any
} as Event<'ChannelUpdated'>

export const TICKET_REDEEMED_EVENT = {
  event: 'TicketRedeemed',
  transactionHash: '',
  blockNumber: 5,
  transactionIndex: 1,
  logIndex: 0,
  args: {
    source: PARTY_A.toAddress().toHex(),
    destination: PARTY_B.toAddress().toHex(),
    nextCommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
    ticketEpoch: BigNumber.from('0'),
    ticketIndex: BigNumber.from('1'),
    proofOfRelaySecret: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
    amount: BigNumber.from('2'),
    winProb: BigNumber.from('1'),
    signature: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex()
  } as any
} as Event<'TicketRedeemed'>

export const oneLargeTicket = new Ticket(
  PARTY_B.toAddress(),
  new Challenge(
    stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10')
  ).toEthereumChallenge(),
  UINT256.fromString('0'),
  UINT256.fromString('0'),
  new Balance(new BN('2')),
  UINT256.fromInverseProbability(new BN(1)),
  UINT256.fromString('0'),
  new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
)
export const oneSmallTicket = new Ticket(
  PARTY_B.toAddress(),
  new Challenge(
    stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10')
  ).toEthereumChallenge(),
  UINT256.fromString('0'),
  UINT256.fromString('0'),
  new Balance(new BN('1')),
  UINT256.fromInverseProbability(new BN(1)),
  UINT256.fromString('0'),
  new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
)

export const PARTY_A_TRANSFER_INCOMING = {
  event: 'Transfer',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    from: PARTY_B.toAddress().toHex(),
    to: PARTY_A.toAddress().toHex(),
    value: BigNumber.from('3')
  } as any
} as TokenEvent<'Transfer'>

export const PARTY_A_TRANSFER_OUTGOING = {
  event: 'Transfer',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    from: PARTY_A.toAddress().toHex(),
    to: PARTY_B.toAddress().toHex(),
    value: BigNumber.from('1')
  } as any
} as TokenEvent<'Transfer'>

export const PARTY_A_ELEGIBLE = {
  event: 'EligibilityUpdated',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A.toAddress().toHex(),
    eligibility: true
  } as any
} as RegistryEvent<'EligibilityUpdated'>

export const PARTY_A_NOT_ELEGIBLE = {
  event: 'EligibilityUpdated',
  transactionHash: '',
  blockNumber: 4,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A.toAddress().toHex(),
    eligibility: false
  } as any
} as RegistryEvent<'EligibilityUpdated'>
