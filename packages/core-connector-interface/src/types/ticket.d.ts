import AccountId from './accountId'
import Balance from './balance'
import Hash from './hash'
import Signature from './signature'
import TicketEpoch from './ticketEpoch'

declare interface TicketStatic {
  readonly SIZE: number

  create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      counterparty: AccountId
      challenge: Hash
      epoch: TicketEpoch
      amount: Balance
      winProb: Hash
      channelIteration: TicketEpoch
    }
  ): Ticket
}
declare interface Ticket {
  counterparty: AccountId
  challenge: Hash
  epoch: TicketEpoch
  amount: Balance
  winProb: Hash
  channelIteration: TicketEpoch

  // computed properties
  hash: Promise<Hash>

  getEmbeddedFunds(): Balance

  toU8a(): Uint8Array

  sign(
    privKey: Uint8Array,
    pubKey: Uint8Array | undefined,
    arr?: {
      bytes: ArrayBuffer
      offset: number
    }
  ): Promise<Signature>
}

declare var Ticket: TicketStatic

export default Ticket
