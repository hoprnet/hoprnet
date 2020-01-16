import { Types } from './types'

export class ChannelClass {
  private constructor()

  readonly channelId: Promise<Types.Hash>

  readonly settlementWindow: Promise<Types.Moment>

  readonly state: Promise<Types.State>

  readonly balance_a: Promise<Types.Balance>

  readonly balance: Promise<Types.Balance>

  readonly currentBalance: Promise<Types.Balance>

  readonly currentBalanceOfCounterparty: Promise<Types.Balance>

  createTicket(secretKey: Uint8Array, amount: Types.Balance, challenge: Types.Hash, winProb: Types.Hash): Promise<Types.Ticket>

  verifyTicket(signedTicket: Types.SignedTicket): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: Types.SignedTicket): Promise<void>
}

export default interface Channel {
  fromDatabase(props: any): Promise<ChannelClass>

  open(amount: Types.Balance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelClass>

  getAllChannels<T, R>(onData: (channelId: Types.Hash, state: Types.State) => T, onEnd: (promises: Promise<T>[]) => R): Promise<R>

  closeChannels(): Promise<Types.Balance>
}
