import { TypeClasses } from './types'

export interface ChannelClass {
  readonly channelId: Promise<TypeClasses.Hash>

  readonly settlementWindow: Promise<TypeClasses.Moment>

  readonly state: Promise<TypeClasses.Channel>

  readonly balance_a: Promise<TypeClasses.Balance>

  readonly balance: Promise<TypeClasses.Balance>

  readonly currentBalance: Promise<TypeClasses.Balance>

  readonly currentBalanceOfCounterparty: Promise<TypeClasses.Balance>

  /**
   * Creates a ticket.
   * @param secretKey the private key of the issuer
   * @param amount how many funds are included
   * @param challenge the challenge that need to resolved to redeem the ticket
   * @param winProb the winning probability
   */
  createTicket(secretKey: Uint8Array, amount: TypeClasses.Balance, challenge: TypeClasses.Hash, winProb: TypeClasses.Hash): Promise<TypeClasses.SignedTicket>

  /**
   * Check `signedTicket` for its validity.
   * @param signedTicket the ticket to check
   */
  verifyTicket(signedTicket: TypeClasses.SignedTicket): Promise<boolean>

  /**
   * Initiates a settlement for this channel.
   */
  initiateSettlement(): Promise<void>

  /**
   * Submits a signed to the blockchain.
   * @param signedTicket a signed ticket
   */
  submitTicket(signedTicket: TypeClasses.SignedTicket): Promise<void>

  /**
   * Fetches all unresolved, previous challenges from the database that
   * have occured in this channel.
   */
  getPreviousChallenges(): Promise<TypeClasses.Hash>
}

export default interface Channel {
  /**
   * Creates a Channel instance from the database.
   * @param props additional arguments
   */
  create(...props: any[]): Promise<ChannelClass>

  /**
   * Opens a new payment channel and initializes the on-chain data.
   * @param amount how much should be staked
   * @param signature a signature over channel state
   * @param props additional arguments
   */
  open(amount: TypeClasses.Balance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelClass>

  /**
   * Fetches all channel instances from the database and applies first `onData` and
   * then `onEnd` on the received nstances.
   * @param onData applied on all channel instances
   * @param onEnd composes at the end the received data
   */
  getAllChannels<T, R>(onData: (channel: ChannelClass, ...props: any[]) => T, onEnd: (promises: Promise<T>[], ...props: any[]) => R, ...props: any[]): Promise<R>

  /**
   * Fetches all channel instances from the database and initiates a settlement on
   * each of them.
   * @param props additional arguments
   */
  closeChannels(...props: any[]): Promise<TypeClasses.Balance>
}
