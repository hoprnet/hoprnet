import { TypeClasses } from './types'

export default class Ticket extends Uint8Array {
  private constructor(...props: any[])

  static create(secretKey: Uint8Array, amount: TypeClasses.Balance, challenge: TypeClasses.Hash, winProb: TypeClasses.Hash): Promise<Ticket>

  static verify(signedTicket: TypeClasses.SignedTicket, ...props: any[]): Promise<boolean>

  static aggregate(tickets: Ticket[], ...props: any[]): Promise<Ticket>
}
