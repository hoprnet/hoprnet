import { TypeClasses } from './types'

export class ChannelClass {
  protected constructor()

  readonly channelId: Promise<TypeClasses.Hash>

  readonly settlementWindow: Promise<TypeClasses.Moment>

  readonly state: Promise<TypeClasses.Channel>

  readonly balance_a: Promise<TypeClasses.Balance>

  readonly balance: Promise<TypeClasses.Balance>

  readonly currentBalance: Promise<TypeClasses.Balance>

  readonly currentBalanceOfCounterparty: Promise<TypeClasses.Balance>

  createTicket(secretKey: Uint8Array, amount: TypeClasses.Balance, challenge: TypeClasses.Hash, winProb: TypeClasses.Hash): Promise<TypeClasses.Ticket>

  verifyTicket(signedTicket: TypeClasses.SignedTicket): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: TypeClasses.SignedTicket): Promise<void>
}

export default interface Channel {
  fromDatabase<U extends ChannelClass>(props: any): Promise<U>

  open<U extends ChannelClass>(amount: TypeClasses.Balance, signature: Promise<Uint8Array>, ...props: any[]): Promise<U>

  getAllChannels<T, R>(onData: (channelId: TypeClasses.Hash, state: TypeClasses.State) => T, onEnd: (promises: Promise<T>[]) => R): Promise<R>

  closeChannels(): Promise<TypeClasses.Balance>
}
