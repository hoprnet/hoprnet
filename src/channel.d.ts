import { TypeClasses } from './types'
import { Ticket } from './ticket'

export interface ChannelClass {
  readonly channelId: Promise<TypeClasses.Hash>

  readonly settlementWindow: Promise<TypeClasses.Moment>

  readonly state: Promise<TypeClasses.Channel>

  readonly balance_a: Promise<TypeClasses.Balance>

  readonly balance: Promise<TypeClasses.Balance>

  readonly currentBalance: Promise<TypeClasses.Balance>

  readonly currentBalanceOfCounterparty: Promise<TypeClasses.Balance>

  ticket: {
    /**
     * Constructs a ticket to use in a probabilistic payment channel.
     * @param secretKey private key of the issuer
     * @param amount amount of funds to include
     * @param challenge a challenge that has to be solved be the redeemer
     * @param winProb winning probability of this ticket
     */
    create(secretKey: Uint8Array, amount: TypeClasses.Balance, challenge: TypeClasses.Hash, winProb: TypeClasses.Hash): Promise<TypeClasses.SignedTicket>

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
    // aggregate(tickets: Ticket[], ...props: any[]): Promise<Ticket>

    /**
     * Submits a signed to the blockchain.
     * @param signedTicket a signed ticket
     */
    submit(signedTicket: TypeClasses.SignedTicket): Promise<void>
  }

  /**
   * Initiates a settlement for this channel.
   */
  initiateSettlement(): Promise<void>

  /**
   * Fetches all unresolved, previous challenges from the database that
   * have occured in this channel.
   */
  getPreviousChallenges(): Promise<TypeClasses.Hash>
}
