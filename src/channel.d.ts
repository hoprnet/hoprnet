import { SignedTicket } from './types'

import { Balance, Hash, Moment, Ticket, State } from './types'

declare class ChannelClass {
  private constructor()

  readonly channelId: Promise<Hash>

  readonly settlementWindow: Promise<Moment>

  readonly state: Promise<State>

  readonly balance_a: Promise<Balance>

  readonly balance: Promise<Balance>

  readonly currentBalance: Promise<Balance>

  readonly currentBalanceOfCounterparty: Promise<Balance>

  createTicket(secretKey: Uint8Array, amount: Balance, challenge: Hash, winProb: Hash): Promise<Ticket>

  verifyTicket(signedTicket: SignedTicket<Ticket>): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: SignedTicket<Ticket>): Promise<void>
}

declare interface ChannelStatic {
  fromDatabase(props: any): Promise<ChannelClass>

  open(
    props: any,
    amount: Balance,
    signature: Promise<Uint8Array>
  ): Promise<ChannelClass>

  getAllChannels<T, R>(
    onData: ({
      channelId: Hash,
      state: State
    }) => T,
    onEnd: (promises: Promise<T>[]) => R
  ): Promise<R>

  closeChannels(): Promise<Balance>
}

declare const Channel: ChannelClass & ChannelStatic

export { ChannelClass, ChannelStatic }

export default Channel