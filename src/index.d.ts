import { LevelUp } from 'levelup'
import BN from 'bn.js'

export interface SignedTicket<Ticket> {
  lotteryTicket: Ticket,
  signature: Uint8Array
}

export interface ChannelStatic<Balance extends BN, Hash extends Uint8Array, Moment, Ticket> {
  new (): Channel<Balance, Hash, Moment, Ticket>

  fromDatabase(props: any): Promise<Channel<Balance, Hash, Moment, Ticket>>

  open(props: any, amount: Balance, signature: Promise<Uint8Array>): Promise<Channel<Balance, Hash, Moment, Ticket>>
}

export interface Channel<Balance extends BN, Hash extends Uint8Array, Moment, Ticket> {
  readonly channelId: Promise<any>

  readonly settlementWindow: Promise<any>

  readonly state: Promise<any>

  readonly balance_a: Promise<Balance>

  readonly balance: Promise<Balance>

  readonly currentBalance: Promise<Balance>

  readonly currentBalanceOfCounterparty: Promise<Balance>

  createTicket(secretKey: Uint8Array, amount: Balance, challenge: Hash, winProb: Hash): Promise<Ticket>

  verifyTicket(signedTicket: SignedTicket<Ticket>): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: SignedTicket<Ticket>): Promise<void>
}

export interface HoprCoreConnectorStatic {
  new (_props: any): HoprCoreConnector

  create(db: LevelUp, keyPair: any, uri: string): Promise<HoprCoreConnector>
}

export default interface HoprCoreConnector {
  started: boolean

  readonly self: any

  readonly db: LevelUp

  nonce: Promise<number>

  /**
   * Creates an uninitialised instance.
   *
   * @param db database instance
   */

  start(): Promise<void>

  stop(): Promise<void>

  initOnchainValues(nonce?: number): Promise<void>

  checkFreeBalance(newBalance: any): Promise<void>
}
