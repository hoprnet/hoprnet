import type BN from 'bn.js'
import type { Public } from '../types'

/**
 * known events we will subscribe and reduce
 * this data ideally should be taken from
 * the web3 types genereted but at this time
 * we use non-standard events
 */
export type EventData = {
  FundedChannel: {
    funder: string
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
  blockNumber: number
  transactionIndex: number
  logIndex: number
  data?: EventData[N]
}
