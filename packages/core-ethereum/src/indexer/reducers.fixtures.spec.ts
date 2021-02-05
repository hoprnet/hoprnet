import type { Event } from './topics'
import BN from 'bn.js'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { Public, ChannelEntry, AccountEntry, AccountId } from '../types'
import { BYTES27_LENGTH } from '../constants'

const partyA = new Public(stringToU8a('0x03767782fdb4564f0a2dee849d9fc356207dd89f195fcfd69ce0b02c6f03dfda40'))
const partyB = new Public(stringToU8a('0x024890561acbe7d1b8832621488a887291eedec2b4bc07a464fef7a9b4c7857cf8'))
const accountId = new AccountId([1])

// channels
export const FUNDED_EVENT: Event<'FundedChannel'> = {
  name: 'FundedChannel',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    funder: accountId,
    recipient: partyA,
    counterparty: partyB,
    recipientAmount: new BN(3),
    counterpartyAmount: new BN(0)
  }
}

export const FUNDED_EVENT_2: Event<'FundedChannel'> = {
  name: 'FundedChannel',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    funder: accountId,
    recipient: partyB,
    counterparty: partyA,
    recipientAmount: new BN(7),
    counterpartyAmount: new BN(0)
  }
}

export const OPENED_EVENT: Event<'OpenedChannel'> = {
  name: 'OpenedChannel',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    opener: partyA,
    counterparty: partyB
  }
}

export const REDEEMED_EVENT: Event<'RedeemedTicket'> = {
  name: 'RedeemedTicket',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    redeemer: partyA,
    counterparty: partyB,
    amount: new BN(1)
  }
}

export const CLOSING_EVENT: Event<'InitiatedChannelClosure'> = {
  name: 'InitiatedChannelClosure',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    initiator: partyA,
    counterparty: partyB,
    closureTime: new BN(1611671775)
  }
}

export const REDEEMED_EVENT_2: Event<'RedeemedTicket'> = {
  name: 'RedeemedTicket',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    redeemer: partyB,
    counterparty: partyA,
    amount: new BN(2)
  }
}

export const CLOSED_EVENT: Event<'ClosedChannel'> = {
  name: 'ClosedChannel',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    closer: partyA,
    counterparty: partyB,
    partyAAmount: new BN(3),
    partyBAmount: new BN(7)
  }
}

export const EMPTY_CHANNEL = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(0),
  partyABalance: new BN(0),
  closureTime: new BN(0),
  stateCounter: new BN(0),
  closureByPartyA: false
})

export const FUNDED_CHANNEL = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(3),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false
})

export const FUNDED_CHANNEL_2 = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(10),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(1),
  closureByPartyA: false
})

export const OPENED_CHANNEL = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(10),
  partyABalance: new BN(3),
  closureTime: new BN(0),
  stateCounter: new BN(2),
  closureByPartyA: false
})

export const REDEEMED_CHANNEL = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(10),
  partyABalance: new BN(4),
  closureTime: new BN(0),
  stateCounter: new BN(2),
  closureByPartyA: false
})

export const CLOSING_CHANNEL = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(10),
  partyABalance: new BN(4),
  closureTime: new BN(1611671775),
  stateCounter: new BN(3),
  closureByPartyA: true
})

export const REDEEMED_CHANNEL_2 = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(10),
  partyABalance: new BN(2),
  closureTime: new BN(1611671775),
  stateCounter: new BN(3),
  closureByPartyA: false
})

export const CLOSED_CHANNEL = new ChannelEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  deposit: new BN(0),
  partyABalance: new BN(0),
  closureTime: new BN(0),
  stateCounter: new BN(10),
  closureByPartyA: false
})

// accounts
export const SECRET_SET_EVENT: Event<'SecretHashSet'> = {
  name: 'SecretHashSet',
  transactionHash: '',
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  data: {
    account: accountId,
    secretHash: new Uint8Array(Buffer.from([1, 2, 3]), undefined, BYTES27_LENGTH),
    counter: new BN(1)
  }
}

export const INITIALIZED_ACCOUNT = new AccountEntry(undefined, {
  blockNumber: new BN(0),
  transactionIndex: new BN(0),
  logIndex: new BN(0),
  hashedSecret: new Uint8Array(Buffer.from([1, 2, 3]), undefined, BYTES27_LENGTH),
  counter: new BN(1)
})
