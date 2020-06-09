import Signature from './signature'
import Ticket from './ticket'

declare namespace SignedTicket {
  const SIZE: number
}
declare interface SignedTicket extends Uint8Array {
  ticket: Ticket
  signature: Signature
  signer: Promise<Uint8Array>

  create(
    arr?: {
      bytes: Uint8Array
      offset: number
    },
    struct?: {
      ticket: Ticket
      signature: Signature
    }
  ): SignedTicket

  verify(pubKey: Uint8Array): Promise<boolean>
}

export default SignedTicket
