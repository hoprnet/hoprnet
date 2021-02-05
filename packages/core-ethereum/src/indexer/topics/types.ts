import type BN from 'bn.js'
import type { Public, AccountId } from '../../types'

export type Topics = (string | string[])[]

/**
 * known event return values that we will subscribe and reduce
 * data from, ideally this should be taken from
 * the web3 types genereted but at this time we can't
 * since we use non-standard events that typechain doesn't
 * recognise
 */
export type EventData = {
  SecretHashSet: {
    account: AccountId
    secretHash: Uint8Array
    counter: BN
  }
  FundedChannel: {
    funder: AccountId
    recipient: Public
    counterparty: Public
    recipientAmount: BN
    counterpartyAmount: BN
  }
  OpenedChannel: {
    opener: Public
    counterparty: Public
  }
  RedeemedTicket: {
    redeemer: Public
    counterparty: Public
    amount: BN
  }
  InitiatedChannelClosure: {
    initiator: Public
    counterparty: Public
    closureTime: BN
  }
  ClosedChannel: {
    closer: Public
    counterparty: Public
    partyAAmount: BN
    partyBAmount: BN
  }
}

/**
 * represents a smart contract event
 */
export type Event<N extends keyof EventData> = {
  name: N
  transactionHash: string
  blockNumber: BN
  transactionIndex: BN
  logIndex: BN
  data?: EventData[N]
}
