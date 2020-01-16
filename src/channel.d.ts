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

  createTicket(secretKey: Uint8Array, amount: TypeClasses.Balance, challenge: TypeClasses.Hash, winProb: TypeClasses.Hash): Promise<TypeClasses.SignedTicket>

  verifyTicket(signedTicket: TypeClasses.SignedTicket): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: TypeClasses.SignedTicket): Promise<void>
}

export default interface Channel {
  fromDatabase(props: any): Promise<ChannelClass>

  open(amount: TypeClasses.Balance, signature: Promise<Uint8Array>, ...props: any[]): Promise<ChannelClass>

  getAllChannels<T, R>(onData: (channelId: TypeClasses.Hash, state: TypeClasses.State) => T, onEnd: (promises: Promise<T>[]) => R): Promise<R>

  closeChannels(): Promise<TypeClasses.Balance>
}
