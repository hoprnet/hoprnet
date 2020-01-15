import Types from './types'

export default class Channel {
  private constructor()

  readonly channelId: Promise<Types['Hash']>

  readonly settlementWindow: Promise<Types['Moment']>

  readonly state: Promise<Types['State']>

  readonly balance_a: Promise<Types['Balance']>

  readonly balance: Promise<Types['Balance']>

  readonly currentBalance: Promise<Types['Balance']>

  readonly currentBalanceOfCounterparty: Promise<Types['Balance']>

  createTicket(secretKey: Uint8Array, amount: Types['Balance'], challenge: Types['Hash'], winProb: Types['Hash']): Promise<Types['Ticket']>

  verifyTicket(signedTicket: Types['SignedTicket']): Promise<boolean>

  initiateSettlement(): Promise<void>

  submitTicket(signedTicket: Types['SignedTicket']): Promise<void>

  static fromDatabase(props: any): Promise<Channel>

  static open(amount: Types['Balance'], signature: Promise<Uint8Array>, ...props: any[]): Promise<Channel>

  static getAllChannels<T, R>(onData: (channelId: Types['Hash'], state: Types['State']) => T, onEnd: (promises: Promise<T>[]) => R): Promise<R>

  static closeChannels(): Promise<Types['Balance']>
}
