import Signature from './signature'
import Ticket from './ticket'

declare interface SignedTicketStatic {
  readonly SIZE: number
  deserialize(Uint8Array): Promise<SignedTicket>
}

declare interface SignedTicket extends Uint8Array {
  constructor(ticket: Ticket, signature: Signature)
  ticket: Ticket
  signature: Signature
  getSigner(): Promise<Uint8Array>
  verifySignature(pubKey: Uint8Array): Promise<boolean>
  serialize(): Uint8Array
}

declare var SignedTicket: SignedTicketStatic

export default SignedTicket
