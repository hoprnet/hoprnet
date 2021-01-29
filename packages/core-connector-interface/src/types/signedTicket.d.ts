import Signature from './signature'
import Ticket from './ticket'

declare interface SignedTicket {
  ticket: Ticket
  signature: Signature
  signer: Promise<Uint8Array>
  verify(pubKey: Uint8Array): Promise<boolean>
  serialize(): Uint8Array
}
declare var SignedTicket
export default SignedTicket
