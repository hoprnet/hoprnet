import { TypeClasses } from './types'

export interface TicketClass extends Uint8Array {}

export default interface Ticket {
  /**
   * Constructs a ticket to use in a probabilistic payment channel.
   * @param secretKey private key of the issuer
   * @param amount amount of funds to include
   * @param challenge a challenge that has to be solved be the redeemer
   * @param winProb winning probability of this ticket
   */
  create(secretKey: Uint8Array, amount: TypeClasses.Balance, challenge: TypeClasses.Hash, winProb: TypeClasses.Hash): Promise<Ticket>

  /**
   * Checks a previously issued ticket for its validity.
   * @param signedTicket a previously issued ticket to check
   * @param props additional arguments
   */
  verify(signedTicket: TypeClasses.SignedTicket, ...props: any[]): Promise<boolean>

  /**
   * BIG TODO
   * Aggregate previously issued tickets. Still under active development!
   * @param tickets array of tickets to aggregate
   * @param props additional arguments
   */
  aggregate(tickets: Ticket[], ...props: any[]): Promise<Ticket>
}
