import AccountId from './accountId'
import Balance from './balance'
import Hash from './hash'
import Signature from './signature'
import TicketEpoch from './ticketEpoch'

declare interface Ticket {
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
declare interface TicketStatic {
  SIZE(): number
  deserialize(Uint8Array): Ticket
}
declare var Ticket: TicketStatic

declare interface SignedTicket {
  ticket: Ticket
  signature: Signature
  getSigner(): Promise<Uint8Array>
  verifySignature(pubKey: Uint8Array): Promise<boolean>
  serialize(): Uint8Array
  toUnacknowledged(secretA: Hash): UnacknowledgedTicket
}
declare interface SignedTicketStatic {
  readonly SIZE: number
  deserialize(Uint8Array): SignedTicket
}
declare var SignedTicket: SignedTicketStatic

declare interface UnacknowledgedTicket {
  signedTicket: SignedTicket
  secretA: Hash
  serialize(): Uint8Array
}
declare interface UnacknowledgedTicketStatic {
  SIZE(): number
  deserialize(Uint8Array): UnacknowledgedTicket
}
declare var UnacknowledgedTicket: UnacknowledgedTicketStatic

/*
 * An acknowledged ticket encapsulates the knowledge we have about a ticket
 * that has been successfully acknowledged by a counterparty, and is waiting
 * for us to redeem it.
 *
 * We don't need to store unsuccessful tickets, or tickets that have been
 * redeemed.
 */
declare interface AcknowledgedTicket {
  getSignedTicket(): SignedTicket
  getResponse(): Hash
  getPreImage(): Hash
  serialize(): Uint8Array
}
declare interface AcknowledgedTicketStatic {
  SIZE(): number
  deserialize(Uint8Array): AcknowledgedTicket
}
declare var AcknowledgedTicket: AcknowledgedTicketStatic

export { Ticket, SignedTicket, UnacknowledgedTicket, AcknowledgedTicket }
