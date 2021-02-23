import AccountId from './accountId'
import Balance from './balance'
import Hash from './hash'
import Signature from './signature'
import TicketEpoch from './ticketEpoch'

declare interface TicketStatic {
  readonly SIZE: number
  deserialize(Uint8Array): Promise<Ticket>
}

declare interface Ticket {
  constructor(
    counterparty: AccountId,
    challenge: Hash,
    epoch: TicketEpoch,
    amount: Balance,
    winProb: Hash,
    channelIteration: TicketEpoch
  )

  counterparty: AccountId
  challenge: Hash
  epoch: TicketEpoch
  amount: Balance
  winProb: Hash
  channelIteration: TicketEpoch

  hash: Promise<Hash>
  getEmbeddedFunds(): Balance
  sign(privKey: Uint8Array): Promise<SignedTicket>
  serialize(): Uint8Array
}

declare var Ticket: TicketStatic

export default Ticket
