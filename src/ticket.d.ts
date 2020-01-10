import { Hash, Balance, SignedTicket } from './types'

declare class TicketClass {
  private constructor(...props: any[])
}

declare interface TicketStatic {
  create(secretKey: Uint8Array, amount: Balance, challenge: Hash, winProb: Hash): Promise<TicketClass>

  verify(signedTicket: SignedTicket<TicketClass>, ...props: any[]): Promise<boolean>

  aggregate(tickets: TicketClass[], ...props: any[]): Promise<TicketClass>
}

declare const Ticket: TicketClass & TicketStatic

export default Ticket
