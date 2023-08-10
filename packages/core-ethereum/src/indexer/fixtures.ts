import type { Event, TokenEvent, RegistryEvent } from './types.js'
import assert from 'assert'
import { BigNumber } from 'ethers'
import {
  Hash,
  AccountEntry,
  ChannelEntry,
  // u8aToHex,
  Ticket,
  Challenge,
  stringToU8a,
  U256,
  Balance,
  BalanceType,
  Signature,
  SIGNATURE_LENGTH
} from '@hoprnet/hopr-utils'
import { PARTY_A, PARTY_B } from '../fixtures.js'

export * from '../fixtures.js'

export const expectAccountsToBeEqual = (actual: AccountEntry, expected: AccountEntry) => {
  assert(actual, 'account is null')
  assert(actual.public_key.eq(expected.public_key), 'publicKey')
}

export const expectChannelsToBeEqual = (actual: ChannelEntry, expected: ChannelEntry) => {
  assert(actual, 'channel is null')
  assert(actual.source.eq(expected.source), 'source')
  assert(actual.destination.eq(expected.destination), 'destination')
  assert(actual.balance.eq(expected.balance), 'balance')
  assert(actual.commitment.eq(expected.commitment), 'commitment')
  assert(actual.ticket_epoch.eq(expected.ticket_epoch), 'ticketEpoch')
  assert(actual.ticket_index.eq(expected.ticket_index), 'ticketIndex')
  assert(actual.status == expected.status, 'status')
  assert(actual.channel_epoch.eq(expected.channel_epoch), 'channelEpoch')
  assert(actual.closure_time.eq(expected.closure_time), 'closureTime')
}

// export const PARTY_A_INITIALIZED_EVENT = {
//   event: 'AddressAnnouncement',
//   transactionHash: '',
//   blockNumber: 1,
//   transactionIndex: 0,
//   logIndex: 0,
//   args: {
//     account: PARTY_A().to_address().to_hex(),
//     publicKey: PARTY_A().to_hex(false),
//     multiaddr: u8aToHex(PARTY_A_MULTIADDR.bytes)
//   }
// } as Event<'AddressAnnouncement'>

// export const PARTY_B_INITIALIZED_EVENT = {
//   event: 'AddressAnnouncement',
//   transactionHash: '',
//   blockNumber: 1,
//   transactionIndex: 1,
//   logIndex: 0,
//   args: {
//     account: PARTY_B().to_address().to_hex(),
//     publicKey: PARTY_B().to_hex(false),
//     multiaddr: u8aToHex(PARTY_B_MULTIADDR.bytes)
//   }
// } as Event<'AddressAnnouncement'>

// TODO LP: Ensure clone here
// export const PARTY_A_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_A(), PARTY_A_MULTIADDR.toString(), 1)

// export const PARTY_B_INITIALIZED_ACCOUNT = new AccountEntry(PARTY_B(), PARTY_B_MULTIADDR.toString(), 1)

export const OPENED_EVENT = {
  event: 'ChannelUpdated',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    source: PARTY_A().to_address().to_hex(),
    destination: PARTY_B().to_address().to_hex(),
    newState: {
      balance: BigNumber.from('3'),
      commitment: new Hash(new Uint8Array({ length: Hash.size() })).to_hex(),
      ticketEpoch: BigNumber.from('0'),
      ticketIndex: BigNumber.from('0'),
      status: 1,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0')
    }
  } as any
} // as Event<'ChannelUpdated'> FIXME:

export const COMMITTED_EVENT = {
  event: 'ChannelUpdated',
  transactionHash: '',
  blockNumber: 2,
  transactionIndex: 0,
  logIndex: 10,
  args: {
    source: PARTY_A().to_address().to_hex(),
    destination: PARTY_B().to_address().to_hex(),
    newState: {
      balance: BigNumber.from('3'),
      commitment: new Hash(new Uint8Array({ length: Hash.size() }).fill(1)).to_hex(),
      ticketEpoch: BigNumber.from('0'),
      ticketIndex: BigNumber.from('0'),
      status: 2,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0')
    }
  } as any
} // as Event<'ChannelUpdated'> FIXME:

export const UPDATED_WHEN_REDEEMED_EVENT = {
  event: 'ChannelUpdated',
  transactionHash: '',
  blockNumber: 5,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    source: PARTY_A().to_address().to_hex(),
    destination: PARTY_B().to_address().to_hex(),
    newState: {
      balance: BigNumber.from('1'),
      commitment: new Hash(new Uint8Array({ length: Hash.size() })).to_hex(),
      ticketEpoch: BigNumber.from('0'),
      ticketIndex: BigNumber.from('1'),
      status: 2,
      channelEpoch: BigNumber.from('0'),
      closureTime: BigNumber.from('0')
    }
  } as any
} // as Event<'ChannelUpdated'> FIXME:

export const TICKET_REDEEMED_EVENT = {
  event: 'TicketRedeemed',
  transactionHash: '',
  blockNumber: 5,
  transactionIndex: 1,
  logIndex: 0,
  args: {
    source: PARTY_A().to_address().to_hex(),
    destination: PARTY_B().to_address().to_hex(),
    nextCommitment: new Hash(new Uint8Array({ length: Hash.size() })).to_hex(),
    ticketEpoch: BigNumber.from('0'),
    ticketIndex: BigNumber.from('1'),
    proofOfRelaySecret: new Hash(new Uint8Array({ length: Hash.size() })).to_hex(),
    amount: BigNumber.from('2'),
    winProb: BigNumber.from('1'),
    signature: new Hash(new Uint8Array({ length: Hash.size() })).to_hex()
  } as any
} as Event<'TicketRedeemed'>

export const oneLargeTicket = new Ticket(
  PARTY_B().to_address(),
  Challenge.deserialize(
    stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10')
  ).to_ethereum_challenge(),
  U256.zero(),
  U256.zero(),
  new Balance('2', BalanceType.HOPR),
  U256.from_inverse_probability(U256.one()),
  U256.zero(),
  new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
)
export const oneSmallTicket = new Ticket(
  PARTY_B().to_address(),
  Challenge.deserialize(
    stringToU8a('0x03c2aa76d6837c51337001c8b5a60473726064fc35d0a40b8f0e1f068cc8e38e10')
  ).to_ethereum_challenge(),
  U256.zero(),
  U256.zero(),
  new Balance('1', BalanceType.HOPR),
  U256.from_inverse_probability(U256.one()),
  U256.zero(),
  new Signature(new Uint8Array({ length: SIGNATURE_LENGTH }), 0)
)

export const PARTY_A_TRANSFER_INCOMING = {
  event: 'Transfer',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    from: PARTY_B().to_address().to_hex(),
    to: PARTY_A().to_address().to_hex(),
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
    from: PARTY_A().to_address().to_hex(),
    to: PARTY_B().to_address().to_hex(),
    value: BigNumber.from('1')
  } as any
} as TokenEvent<'Transfer'>

export const REGISTER_ENABLED = {
  event: 'EnabledNetworkRegistry',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    isEnabled: true
  } as any
} as RegistryEvent<'EnabledNetworkRegistry'>

export const REGISTER_DISABLED = {
  event: 'EnabledNetworkRegistry',
  transactionHash: '',
  blockNumber: 3,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    isEnabled: false
  } as any
} as RegistryEvent<'EnabledNetworkRegistry'>

export const PARTY_A_REGISTERED = {
  event: 'Registered',
  transactionHash: '',
  blockNumber: 1,
  transactionIndex: 1,
  logIndex: 0,
  args: {
    account: PARTY_A().to_address().to_hex(),
    hoprPeerId: PARTY_B().to_peerid_str()
  } as any
} as RegistryEvent<'Registered'>

export const PARTY_A_ELEGIBLE = {
  event: 'EligibilityUpdated',
  transactionHash: '',
  blockNumber: 3,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A().to_address().to_hex(),
    eligibility: true
  } as any
} as RegistryEvent<'EligibilityUpdated'>

export const PARTY_A_NOT_ELEGIBLE = {
  event: 'EligibilityUpdated',
  transactionHash: '',
  blockNumber: 5,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A().to_address().to_hex(),
    eligibility: false
  } as any
} as RegistryEvent<'EligibilityUpdated'>

export const PARTY_A_ELEGIBLE_2 = {
  event: 'EligibilityUpdated',
  transactionHash: '',
  blockNumber: 7,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A().to_address().to_hex(),
    eligibility: true
  } as any
} as RegistryEvent<'EligibilityUpdated'>

export const PARTY_A_DEREGISTERED = {
  event: 'DeregisteredByOwner',
  transactionHash: '',
  blockNumber: 9,
  transactionIndex: 0,
  logIndex: 0,
  args: {
    account: PARTY_A().to_address().to_hex(),
    hoprPeerId: PARTY_B().to_peerid_str()
  } as any
} as RegistryEvent<'DeregisteredByOwner'>
