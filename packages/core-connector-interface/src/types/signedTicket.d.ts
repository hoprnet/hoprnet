import Signature from './signature'
import Ticket from './ticket'

declare interface SignedTicketStatic {
  readonly SIZE: number
  deserialize(Uint8Array): SignedTicket
}

declare interface SignedTicket {
  //constructor(ticket: Ticket, signature: Signature): SignedTicket
  ticket: Ticket
  signature: Signature
  getSigner(): Promise<Uint8Array>
  verifySignature(pubKey: Uint8Array): Promise<boolean>
  serialize(): Uint8Array
}

declare var SignedTicket: SignedTicketStatic

export default SignedTicket
