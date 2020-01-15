import Types from './types'

export default class Ticket extends Uint8Array {
  private constructor(...props: any[])

  static create(secretKey: Uint8Array, amount: Types['Balance'], challenge: Types['Hash'], winProb: Types['Hash']): Promise<Ticket>

  static verify(signedTicket: Types['SignedTicket'], ...props: any[]): Promise<boolean>

  static aggregate(tickets: Ticket[], ...props: any[]): Promise<Ticket>
}