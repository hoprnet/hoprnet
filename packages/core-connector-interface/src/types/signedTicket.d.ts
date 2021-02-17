import Signature from './signature'
import Ticket from './ticket'

declare interface SignedTicketStatic {
  readonly SIZE: number

  create(
    arr?: {
      bytes: ArrayBuffer
      offset: number
    },
    struct?: {
      ticket?: Ticket
      signature?: Signature
    }
  ): Promise<SignedTicket>
}

declare interface SignedTicket extends Uint8Array {
  ticket: Ticket
  signature: Signature
  signer: Promise<Uint8Array>
  ticketOffset: number
  signatureOffset: number

  verifySignature(pubKey: Uint8Array): Promise<boolean>
}

declare var SignedTicket: SignedTicketStatic

export default SignedTicket
