import { SignedTicket } from './types'

import { Balance, Hash, Moment, Ticket, State } from './types'

export default class Channel {
  private constructor()

  readonly channelId: Promise<Hash>

  readonly settlementWindow: Promise<Moment>

  readonly state: Promise<State>

  readonly balance_a: Promise<Balance>

  readonly balance: Promise<Balance>

  readonly currentBalance: Promise<Balance>

  readonly currentBalanceOfCounterparty: Promise<Balance>

  createTicket(secretKey: Uint8Array, amount: Balance, challenge: Hash, winProb: Hash): Promise<Ticket>

  verifyTicket(signedTicket: SignedTicket): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: SignedTicket): Promise<void>

  static fromDatabase(props: any): Promise<Channel>

  static open(
    props: any,
    amount: Balance,
    signature: Promise<Uint8Array>
  ): Promise<Channel>

  static getAllChannels<T, R>(
    onData: ({
      channelId: Hash,
      state: State
    }) => T,
    onEnd: (promises: Promise<T>[]) => R
  ): Promise<R>

  static closeChannels(): Promise<Balance>
}